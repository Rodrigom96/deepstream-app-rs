use anyhow::Error;
use gst::prelude::*;
use log::{debug, error, info, warn};

use super::super::common;
use super::super::common::MissingElement;

use super::base_source::Source;

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
        rtspsrc.set_property("user-id", "admin")?;
        rtspsrc.set_property("user-pw", "Ogi.1796")?;

        // Add elements to bin
        bin.add_many(&[&rtspsrc, &decodebin, &queue])?;

        // Add bin sink ghostpad
        common::add_bin_ghost_pad(&bin, &queue, "src")?;

        // Connect the pad-added signal
        let decodebin_weak = decodebin.downgrade();
        rtspsrc.connect_pad_added(move |src, src_pad| {
            debug!("Received new pad {} from {}", src_pad.name(), src.name());
            let decodebin = match decodebin_weak.upgrade() {
                Some(decodebin) => decodebin,
                None => return,
            };

            let sink_pad = decodebin
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                warn!("We are already linked. Ignoring.");
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
            debug!("Received pad type {}", new_pad_type);

            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                error!("Type is {} but link failed.", new_pad_type);
            } else {
                info!("Link succeeded (type {}).", new_pad_type);
            }
        });

        let queue_weak = queue.downgrade();
        decodebin.connect_pad_added(move |src, src_pad| {
            debug!("Received new pad {} from {}", src_pad.name(), src.name());
            let queue = match queue_weak.upgrade() {
                Some(queue) => queue,
                None => return,
            };

            let sink_pad = queue
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                warn!("We are already linked. Ignoring.");
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
            debug!("Received pad type {}", new_pad_type);
            
            let is_video = new_pad_type.starts_with("video/");
            if !is_video {
                debug!("It has type {} which is not video. Ignoring.", new_pad_type);
                return;
            }

            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                error!("Type is {} but link failed.", new_pad_type);
            } else {
                info!("Link succeeded (type {}).", new_pad_type);
            }
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
