use anyhow::Error;
use gst::prelude::*;
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::common;
use super::super::common::MissingElement;

use super::base_source::Source;

fn pad_add_handler(src: &gst::Element, src_pad: &gst::Pad, sink: &gst::Element) {
    debug!(
        "Received new pad {} from {} to sink {}",
        src_pad.name(),
        src.name(),
        sink.name()
    );

    let sink_pad = sink
        .static_pad("sink")
        .expect("Failed to get static sink pad from convert");
    if sink_pad.is_linked() {
        warn!("{} already linked. Ignoring.", sink.name());
        return;
    }
    let new_pad_caps = match src_pad.current_caps() {
        Some(cap) => cap,
        None => src_pad.query_caps(None),
    };
    let new_pad_struct = new_pad_caps
        .structure(0)
        .expect("Failed to get first structure of caps.");
    let new_pad_type = new_pad_struct.name();
    debug!("Received pad type {} from {}", new_pad_type, src.name());

    let res = src_pad.link(&sink_pad);
    if res.is_err() {
        error!("Type is {} but link failed.", new_pad_type);
    } else {
        debug!("Link succeeded (type {}).", new_pad_type);
    }
}

fn watch_source_async_state_change(bin: &gst::Bin, ctx: &Arc<Mutex<ReconectionContext>>) -> bool {
    let (ret, state, pending) = bin.state(gst::ClockTime::ZERO);

    log::debug!(
        "Bin {}: state:{:?} pending:{:?} ret:{:?}",
        bin.name(),
        state,
        pending,
        ret
    );

    // Bin is still changing state ASYNC. Wait for some more time.
    if let Ok(success) = ret {
        if success == gst::StateChangeSuccess::Async {
            return true;
        }
    }

    // Bin state change failed / failed to get state
    if ret.is_err() {
        let mut ctx = ctx.lock().unwrap();
        ctx.async_state_watch_timeout = None;
        return false;
    }

    // Bin successfully changed state to PLAYING. Stop watching state
    if state == gst::State::Playing {
        let mut ctx = ctx.lock().unwrap();
        ctx.reconecting = false;
        ctx.async_state_watch_timeout = None;
        return false;
    }

    // Bin has stopped ASYNC state change but has not gone into
    // PLAYING. Expliclity set state to PLAYING and keep watching
    // state
    bin.set_state(gst::State::Playing)
        .expect("Error set bin state to playing");

    true
}

fn reset_source_bin(
    bin: &gst::Bin,
    ctx: &Arc<Mutex<Contex>>,
    reconnection_ctx: &Arc<Mutex<ReconectionContext>>,
) {
    if bin.set_state(gst::State::Null).is_err() {
        log::error!("Cant set source bin {} state to NULL", bin.name());
        return;
    }

    log::debug!("Resetting source {}", bin.name());

    if bin.sync_state_with_parent().is_err() {
        log::error!("Cant sync state with parent of source {}", bin.name());
    }

    if let Some(parser) = &ctx.lock().unwrap().parser {
        if !parser.send_event(ds::events::new_stream_reset(0).unwrap()) {
            log::error!("Interrupted, Reconnection event not sent");
        }
    }

    let (ret, state, pending) = bin.state(gst::ClockTime::ZERO);

    log::debug!(
        "Bin {}: state:{:?} pending:{:?} ret:{:?}",
        bin.name(),
        state,
        pending,
        ret
    );

    if let Ok(success) = ret {
        if success == gst::StateChangeSuccess::Async
            || success == gst::StateChangeSuccess::NoPreroll
        {
            let bin_week = bin.downgrade();
            let reconnection_ctx_clone = reconnection_ctx.clone();
            let timeout_id = glib::timeout_add(std::time::Duration::from_millis(20), move || {
                let bin = bin_week.upgrade().unwrap();
                let ret = watch_source_async_state_change(&bin, &reconnection_ctx_clone);
                glib::Continue(ret)
            });
            {
                let mut reconnection_ctx_lock = reconnection_ctx.lock().unwrap();
                reconnection_ctx_lock.async_state_watch_timeout = Some(timeout_id);
            }
        } else if success == gst::StateChangeSuccess::Success && state == gst::State::Playing {
            let mut reconnection_ctx_lock = reconnection_ctx.lock().unwrap();
            reconnection_ctx_lock.reconecting = false;
        }
    };
}

struct Contex {
    depay: Option<gst::Element>,
    parser: Option<gst::Element>,
}

impl Contex {
    pub fn new() -> Self {
        Self {
            depay: None,
            parser: None,
        }
    }
}

struct ReconectionContext {
    last_buffer_update: Instant,
    last_reset_time: Instant,
    started: bool,
    reconecting: bool,
    last_reconnect_time: Instant,
    have_eos: bool,
    async_state_watch_timeout: Option<glib::SourceId>,
}

impl ReconectionContext {
    pub fn new() -> Self {
        let last_buffer_update = Instant::now();
        let last_reset_time = Instant::now();
        let started = false;
        let reconecting = false;
        let last_reconnect_time = Instant::now();
        let have_eos = false;

        Self {
            last_buffer_update,
            last_reset_time,
            started,
            reconecting,
            last_reconnect_time,
            have_eos,
            async_state_watch_timeout: None,
        }
    }
}

pub struct RTSPSource {
    bin: gst::Bin,
    source_watch_timeout: Option<glib::SourceId>,
    reconnection_ctx: Arc<Mutex<ReconectionContext>>,
}

impl RTSPSource {
    pub fn new(
        uri: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self, Error> {
        let bin = gst::Bin::new(None);

        let rtspsrc =
            gst::ElementFactory::make("rtspsrc", None).map_err(|_| MissingElement("rtspsrc"))?;
        let decodebin = gst::ElementFactory::make("decodebin", None)
            .map_err(|_| MissingElement("decodebin"))?;
        let queue =
            gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;

        let ctx = Arc::new(Mutex::new(Contex::new()));
        let reconnection_ctx = Arc::new(Mutex::new(ReconectionContext::new()));

        // Config rtspsrc
        rtspsrc.set_property("location", &uri)?;
        rtspsrc.set_property("latency", 100_u32)?;
        rtspsrc.set_property("drop-on-latency", true)?;

        // Config rtsp auth
        if let Some(username) = username {
            rtspsrc.set_property("user-id", &username)?;
        }
        if let Some(password) = password {
            rtspsrc.set_property("user-pw", &password)?;
        }

        // Add elements to bin
        bin.add_many(&[&rtspsrc, &decodebin, &queue])?;

        // Add bin sink ghostpad
        common::add_bin_ghost_pad(&bin, &queue, "src")?;

        // Only select video stream
        let ctx_clone = ctx.clone();
        let bin_week = bin.downgrade();
        let decodebin_week = decodebin.downgrade();
        rtspsrc.connect("select-stream", false, move |args| {
            let caps = args[2].get::<gst::Caps>().unwrap();
            let caps_struct = caps.structure(0).expect("Failed to get structure of caps.");
            let media: String = caps_struct
                .get("media")
                .expect("error on get struct \"media\"");
            let encoding_name: String = caps_struct
                .get("encoding-name")
                .expect("error on get struct \"encoding-name\"");

            let is_video = media == "video";
            if !is_video {
                return Some(false.to_value());
            }

            // get and lock decoder
            let mut ctx = ctx_clone.lock().unwrap();

            // Create and add depay and parser if not created yet
            if ctx.depay.is_none() {
                let (depay, parser) = match encoding_name.as_str() {
                    "H264" => {
                        let depay = gst::ElementFactory::make("rtph264depay", None)
                            .expect("Cant create \"rtph264depay\" element");
                        let parser = gst::ElementFactory::make("h264parse", None)
                            .expect("Cant create \"h264parse\" element");
                        (depay, parser)
                    }
                    "H265" => {
                        let depay = gst::ElementFactory::make("rtph265depay", None)
                            .expect("Cant create \"rtph265depay\" element");
                        let parser = gst::ElementFactory::make("h265parse", None)
                            .expect("Cant create \"h265parse\" element");
                        (depay, parser)
                    }
                    _ => {
                        log::warn!("{} not supported", encoding_name);
                        return Some(false.to_value());
                    }
                };
                // add elements to bin
                bin_week
                    .upgrade()
                    .unwrap()
                    .add_many(&[&depay, &parser])
                    .expect("Cant add depay and parser");

                // link elements
                depay.link(&parser).expect("Cant link depay with parser");
                let decodebin = decodebin_week.upgrade().unwrap();
                parser
                    .link(&decodebin)
                    .expect("Cant link parser with decodebin");

                // sync elements with pipeline
                depay
                    .sync_state_with_parent()
                    .expect("Depay, Cant sync state with parent");
                parser
                    .sync_state_with_parent()
                    .expect("Parser, Cant sync state with parent");

                // store depay on decoder
                ctx.depay = Some(depay);
                ctx.parser = Some(parser);
            }
            Some(true.to_value())
        })?;

        // Connect the pad-added signal
        //let bin_week = bin.downgrade();
        let ctx_clone = ctx.clone();
        let reconection_ctx_clone = reconnection_ctx.clone();
        rtspsrc.connect_pad_added(move |src, src_pad| {
            let reconection_ctx_clone2 = reconection_ctx_clone.clone();
            {
                let mut ctx = reconection_ctx_clone2.lock().unwrap();
                ctx.started = true;
            }
            src_pad.add_probe(
                gst::PadProbeType::BUFFER | gst::PadProbeType::EVENT_DOWNSTREAM,
                move |_, info| {
                    //let bin = bin_week.upgrade().unwrap();

                    match &info.data {
                        Some(gst::PadProbeData::Buffer(_)) => {
                            {
                                let mut ctx = reconection_ctx_clone2.lock().unwrap();
                                ctx.last_buffer_update = Instant::now();
                                ctx.have_eos = false;
                            }
                            //log::debug!("Update buffer");
                        }
                        Some(gst::PadProbeData::Event(event)) => {
                            let t = event.type_();
                            if t == gst::EventType::Eos {
                                let mut ctx = reconection_ctx_clone2.lock().unwrap();
                                ctx.have_eos = true;
                                log::debug!("prob EOS");
                            }
                        }
                        _ => {}
                    };

                    gst::PadProbeReturn::Ok
                },
            );
            pad_add_handler(
                src,
                src_pad,
                ctx_clone.lock().unwrap().depay.as_ref().unwrap(),
            );
        });

        let queue_weak = queue.downgrade();
        decodebin.connect_pad_added(move |src, src_pad| {
            let queue = match queue_weak.upgrade() {
                Some(queue) => queue,
                None => return,
            };
            pad_add_handler(src, src_pad, &queue);
        });

        let reconnection_ctx_clone = reconnection_ctx.clone();
        let bin_week = bin.downgrade();
        let source_watch_timeout = Some(glib::timeout_add(
            std::time::Duration::from_secs(1),
            move || {
                let bin = bin_week.upgrade().unwrap();
                let reset_requierd = {
                    let reconnection_ctx = reconnection_ctx_clone.lock().unwrap();
                    let update_elapsed = reconnection_ctx.last_buffer_update.elapsed();
                    let reset_elapsed = reconnection_ctx.last_reset_time.elapsed();

                    if reconnection_ctx.reconecting {
                        if reconnection_ctx.last_reconnect_time.elapsed() >= Duration::from_secs(30)
                        {
                            log::warn!("Reconect failed from source {}, trying again", bin.name());
                            true
                        } else {
                            false
                        }
                    } else if reconnection_ctx.started
                        && update_elapsed >= Duration::from_secs(10)
                        && reset_elapsed >= Duration::from_secs(10)
                    {
                        log::warn!("No data from source {}, trying reconect", bin.name());
                        true
                    } else {
                        false
                    }
                };

                if reset_requierd {
                    log::debug!("Reset source");
                    {
                        let mut reconnection_ctx = reconnection_ctx_clone.lock().unwrap();
                        reconnection_ctx.reconecting = true;
                        reconnection_ctx.last_reconnect_time = Instant::now();
                    }
                    reset_source_bin(&bin, &ctx, &reconnection_ctx_clone);
                }

                glib::Continue(true)
            },
        ));

        Ok(RTSPSource {
            bin,
            source_watch_timeout,
            reconnection_ctx,
        })
    }
}

impl Source for RTSPSource {
    fn link(&self, dst: &gst::Element) -> Result<(), Error> {
        self.bin.link(dst)?;

        Ok(())
    }

    fn get_bin(&self) -> &gst::Bin {
        &self.bin
    }

    fn on_remove(&mut self) {
        let mut reconnection_ctx = self.reconnection_ctx.lock().unwrap();

        // stop glib timeouts
        if let Some(source_id) = self.source_watch_timeout.take() {
            glib::source_remove(source_id);
        };
        if let Some(source_id) = reconnection_ctx.async_state_watch_timeout.take() {
            glib::source_remove(source_id);
        }
        log::debug!("Timeouts deleted")
    }
}
