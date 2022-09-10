use anyhow::Error;
use gst::prelude::*;
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};

use super::super::common;
use super::super::common::MissingElement;

use super::base_source::Source;

fn pad_add_handler(src: &gst::Element, src_pad: &gst::Pad, sink: &gst::Element) {
    debug!("Received new pad {} from {} to sink {}", src_pad.name(), src.name(), sink.name());

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

struct Decoder {
    depay: Option<gst::Element>,
}

pub struct RTSPSource {
    bin: gst::Bin,
}

impl RTSPSource {
    pub fn new(uri: String) -> Result<Self, Error> {
        let bin = gst::Bin::new(None);

        let rtspsrc =
            gst::ElementFactory::make("rtspsrc", None).map_err(|_| MissingElement("rtspsrc"))?;
        let decodebin = gst::ElementFactory::make("decodebin", None)
            .map_err(|_| MissingElement("decodebin"))?;
        let queue =
            gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;

        let decoder = Arc::new(Mutex::new(Decoder{depay: None}));

        // Config rtspsrc
        rtspsrc.set_property("location", &uri)?;
        rtspsrc.set_property("latency", 100_u32)?;
        rtspsrc.set_property("drop-on-latency", true)?;

        // Add elements to bin
        bin.add_many(&[&rtspsrc, &decodebin, &queue])?;

        // Add bin sink ghostpad
        common::add_bin_ghost_pad(&bin, &queue, "src")?;

        // Only select video stream
        let decoder_clone = decoder.clone();
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
                    return Some(false.to_value())
                }
            };
            // add elements to bin
            bin_week.upgrade().unwrap().add_many(&[&depay, &parser]).expect("Cant add depay and parser");

            // link elements
            depay.link(&parser).expect("Cant link depay with parser");
            let decodebin = decodebin_week.upgrade().unwrap();
            parser.link(&decodebin).expect("Cant link parser with decodebin");

            // get and lock decoder
            let mut decoder = decoder_clone.lock().unwrap();

            // sync elements with pipeline
            depay.sync_state_with_parent().expect("Depay, Cant sync state with parent");
            parser.sync_state_with_parent().expect("Parser, Cant sync state with parent");

            // store depay on decoder
            decoder.depay = Some(depay);
            Some(true.to_value())
        })?;

        // Connect the pad-added signal
        rtspsrc.connect_pad_added(move |src, src_pad| {
            pad_add_handler(src, src_pad, decoder.lock().unwrap().depay.as_ref().unwrap());
        });

        let queue_weak = queue.downgrade();
        decodebin.connect_pad_added(move |src, src_pad| {
            let queue = match queue_weak.upgrade() {
                Some(queue) => queue,
                None => return,
            };
            pad_add_handler(src, src_pad, &queue);
        });

        Ok(RTSPSource { bin })
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
}
