use anyhow::Error;
use gst::prelude::*;
use log::{debug, error, info, warn};

use super::common;
use super::common::MissingElement;

pub struct TestSource {
    pub bin: gst::Bin,
}

pub trait Source {
    fn get_bin(&self) -> &gst::Bin;
    fn link(&self, dst: &gst::Element) -> Result<(), Error>;
}

impl TestSource {
    pub fn new() -> Result<Self, Error> {
        let bin = gst::Bin::new(None);

        let src = gst::ElementFactory::make("videotestsrc", None)
            .map_err(|_| MissingElement("videotestsrc"))?;

        bin.add_many(&[&src])?;
        common::add_bin_ghost_pad(&bin, &src, "src")?;

        Ok(TestSource { bin })
    }
}

impl Source for TestSource {
    fn link(&self, dst: &gst::Element) -> Result<(), Error> {
        self.bin.link(dst).expect("Failed to link TestSourve");

        Ok(())
    }

    fn get_bin(&self) -> &gst::Bin {
        &self.bin
    }
}

pub struct URISource {
    bin: gst::Bin,
}

impl URISource {
    pub fn new(uri: String) -> Result<Self, Error> {
        let bin = gst::Bin::new(None);

        let urisrc = gst::ElementFactory::make("uridecodebin", None)
            .map_err(|_| MissingElement("uridecodebin"))?;
        let queue =
            gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
        // Config urisourcebin
        urisrc.set_property("uri", &uri)?;

        // Add elements to queue
        bin.add_many(&[&urisrc, &queue])?;

        // Add bin sink ghostpad
        common::add_bin_ghost_pad(&bin, &queue, "src")?;

        // Connect the pad-added signal
        let queue_weak = queue.downgrade();
        urisrc.connect_pad_added(move |src, src_pad| {
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
            let is_video = new_pad_type.starts_with("video/x-raw");
            if !is_video {
                debug!("It has type {} which is not video. Ignoring.", new_pad_type);
                return;
            }

            let features = new_pad_caps.features(0).unwrap();
            if !features.contains("memory:NVMM") {
                panic!("Feature {} not contain 'memory:NVMM'.", features);
            }

            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                error!("Type is {} but link failed.", new_pad_type);
            } else {
                info!("Link succeeded (type {}).", new_pad_type);
            }
        });

        Ok(URISource { bin })
    }
}

impl Source for URISource {
    fn link(&self, dst: &gst::Element) -> Result<(), Error> {
        self.bin.link(dst)?;

        Ok(())
    }

    fn get_bin(&self) -> &gst::Bin {
        &self.bin
    }
}
