use anyhow::Error;
use gst::prelude::*;

use super::common;
use super::config::SinksConfig;
use common::MissingElement;

mod msg_broker;
mod render_sink;
mod rtsp_sink;

pub fn create_sink_bin(config: SinksConfig) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(Some("sink_bin"));

    let queue = gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
    let tee = gst::ElementFactory::make("tee", None).map_err(|_| MissingElement("tee"))?;
    bin.add_many(&[&queue, &tee])?;
    queue.link(&tee)?;

    // Add kafka msg broker
    if let Some(broker_config) = config.msg_broker {
        let broker = msg_broker::create_bin(Some("kafkamsgbroker_sink"), broker_config)?;
        bin.add(&broker)?;
        common::link_element_to_tee_src_pad(&tee, &broker)?;
    }

    // Add filter to proccess images for display
    let display_queue =
        gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
    let nvvidconv = gst::ElementFactory::make("nvvideoconvert", None)
        .map_err(|_| MissingElement("nvvideoconvert"))?;
    let nvosd =
        gst::ElementFactory::make("nvdsosd", None).map_err(|_| MissingElement("nvdsosd"))?;
    let tiler = gst::ElementFactory::make("nvmultistreamtiler", None)
        .map_err(|_| MissingElement("nvmultistreamtiler"))?;
    let tee_display = gst::ElementFactory::make("tee", None).map_err(|_| MissingElement("tee"))?;
    bin.add_many(&[&display_queue, &nvvidconv, &nvosd, &tiler, &tee_display])?;
    common::link_element_to_tee_src_pad(&tee, &display_queue)?;
    display_queue.link(&nvvidconv)?;
    nvvidconv.link(&nvosd)?;
    nvosd.link(&tiler)?;
    tiler.link(&tee_display)?;

    // Add display sinks
    if config.display {
        let render_sink = render_sink::create_bin(Some("render_sink"))?;
        bin.add(&render_sink)?;
        common::link_element_to_tee_src_pad(&tee_display, &render_sink)?;
    }

    // Add rtsp sink
    if let Some(rtsp_config) = config.rtsp {
        let rtsp_sink =
            rtsp_sink::create_bin(Some("rtsp_sink"), &rtsp_config.path, rtsp_config.port)?;
        bin.add(&rtsp_sink)?;
        common::link_element_to_tee_src_pad(&tee_display, &rtsp_sink)?;
    }

    common::add_bin_ghost_pad(&bin, &queue, "sink")?;
    Ok(bin)
}
