use anyhow::Error;
use gst::prelude::*;

use super::common;
use common::MissingElement;

mod render_sink;

pub fn create_sink_bin(display: bool) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(Some("sink_bin"));

    let queue = gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
    let tee = gst::ElementFactory::make("tee", None).map_err(|_| MissingElement("tee"))?;
    
    bin.add_many(&[&queue, &tee])?;
    queue.link(&tee)?;

    if display {
        // Add filter to proccess images for display
        let nvvidconv = gst::ElementFactory::make("nvvideoconvert", None)
            .map_err(|_| MissingElement("nvvideoconvert"))?;
        let tiler = gst::ElementFactory::make("nvmultistreamtiler", None)
            .map_err(|_| MissingElement("nvmultistreamtiler"))?;
        let tee_display = gst::ElementFactory::make("tee", None).map_err(|_| MissingElement("tee"))?;
        bin.add_many(&[&nvvidconv, &tiler, &tee_display])?;
        common::link_element_to_tee_src_pad(&tee, &nvvidconv)?;
        nvvidconv.link(&tiler)?;
        tiler.link(&tee_display)?;

        // Add sinks
        let render_sink = render_sink::create_bin(Some("render_sink"))?;
        bin.add(&render_sink)?;
        common::link_element_to_tee_src_pad(&tee_display, &render_sink)?;
    } else {
        let sink =
            gst::ElementFactory::make("fakesink", None).map_err(|_| MissingElement("fakesink"))?;
        bin.add(&sink)?;
        common::link_element_to_tee_src_pad(&tee, &sink)?;
    }

    common::add_bin_ghost_pad(&bin, &queue, "sink")?;

    Ok(bin)
}
