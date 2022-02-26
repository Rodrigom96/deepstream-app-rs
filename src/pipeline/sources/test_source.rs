use anyhow::Error;
use gst::prelude::*;

use super::super::common;
use super::super::common::MissingElement;

use super::base_source::Source;

pub struct TestSource {
    pub bin: gst::Bin,
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
