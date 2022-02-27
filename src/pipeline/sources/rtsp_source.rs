use anyhow::Error;
use gst::prelude::*;
use log::{debug, error, warn};

use super::super::common;
use super::super::common::MissingElement;

use super::base_source::Source;

fn pad_add_handler(src: &gst::Element, src_pad: &gst::Pad, sink: &gst::Element) {
    debug!("Received new pad {} from {}", src_pad.name(), src.name());

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

        // Config rtspsrc
        rtspsrc.set_property("location", &uri)?;
        rtspsrc.set_property("latency", 100_u32)?;
        rtspsrc.set_property("drop-on-latency", true)?;

        // Add elements to bin
        bin.add_many(&[&rtspsrc, &decodebin, &queue])?;

        // Add bin sink ghostpad
        common::add_bin_ghost_pad(&bin, &queue, "src")?;

        // Only select video stream
        rtspsrc.connect("select-stream", false, |args| {
            let caps = args[2].get::<gst::Caps>().unwrap();
            let caps_struct = caps.structure(0).expect("Failed to get structure of caps.");
            let is_video = caps_struct.to_string().contains("media=(string)video");
            Some(is_video.to_value())
        })?;

        // Connect the pad-added signal
        let decodebin_weak = decodebin.downgrade();
        rtspsrc.connect_pad_added(move |src, src_pad| {
            let decodebin = match decodebin_weak.upgrade() {
                Some(decodebin) => decodebin,
                None => return,
            };
            pad_add_handler(src, src_pad, &decodebin);
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
