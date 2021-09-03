use anyhow::Error;
use gst::prelude::*;

use super::super::common;
use common::MissingElement;

/// Return a bin with nveglglessink
pub fn create_bin(name: Option<&str>) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(name);

    let queue = gst::ElementFactory::make("queue", None)
    .map_err(|_| MissingElement("queue"))?;
    let sink = gst::ElementFactory::make("nveglglessink", None)
        .map_err(|_| MissingElement("nveglglessink"))?;

    bin.add_many(&[&queue,&sink])?;
    common::add_bin_ghost_pad(&bin, &queue, "sink")?;
    queue.link(&sink)?;

    Ok(bin)
}
