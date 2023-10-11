use anyhow::Error;
use gst::prelude::*;

use super::super::common;
use common::MissingElement;

/// Return a bin with nveglglessink
pub fn create_bin(name: Option<&str>) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(name);

    let queue = gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
    let tiler = gst::ElementFactory::make("nvmultistreamtiler", None)
        .map_err(|_| MissingElement("nvmultistreamtiler"))?;
    let sink = gst::ElementFactory::make("nveglglessink", None)
        .map_err(|_| MissingElement("nveglglessink"))?;

    bin.add_many(&[&queue, &tiler, &sink])?;
    common::add_bin_ghost_pad(&bin, &queue, "sink")?;
    queue.link(&tiler)?;
    tiler.link(&sink)?;

    Ok(bin)
}
