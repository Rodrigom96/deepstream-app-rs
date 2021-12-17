use anyhow::Error;
use derive_more::{Display, Error};
use gst::prelude::*;
use gst_rtsp_server::prelude::*;
use log::info;

use super::super::common;
use common::MissingElement;

enum EncoderType {
    H264,
    H265,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Could not get mount points")]
struct NoMountPoints;

static mut RTSP_SERVERS: Vec<gst_rtsp_server::RTSPServer> = Vec::new();

pub fn create_bin(name: Option<&str>) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(name);

    let queue = gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;

    let transform = gst::ElementFactory::make("nvvideoconvert", None)
        .map_err(|_| MissingElement("nvvideoconvert"))?;

    let cap_filter =
        gst::ElementFactory::make("capsfilter", None).map_err(|_| MissingElement("capsfilter"))?;
    let caps = gst::Caps::builder("video/x-raw")
        .features(&["memory:NVMM"])
        .field("format", "I420")
        .build();
    cap_filter.set_property("caps", &caps)?;

    let codecparse =
        gst::ElementFactory::make("h264parse", None).map_err(|_| MissingElement("h264parse"))?;

    let rtppay =
        gst::ElementFactory::make("rtph264pay", None).map_err(|_| MissingElement("rtph264pay"))?;

    let encoder = gst::ElementFactory::make("nvv4l2h264enc", None)
        .map_err(|_| MissingElement("nvv4l2h264enc"))?;

    let sink = gst::ElementFactory::make("udpsink", None).map_err(|_| MissingElement("udpsink"))?;
    sink.set_property("host", "224.224.255.255")?;
    sink.set_property("port", 5400)?;
    sink.set_property("async", false)?;
    sink.set_property("sync", false)?;

    bin.add_many(&[
        &queue,
        &transform,
        &cap_filter,
        &encoder,
        &codecparse,
        &rtppay,
        &sink,
    ])?;
    queue.link(&transform)?;
    transform.link(&cap_filter)?;
    cap_filter.link(&encoder)?;
    encoder.link(&codecparse)?;
    codecparse.link(&rtppay)?;
    rtppay.link(&sink)?;

    common::add_bin_ghost_pad(&bin, &queue, "sink")?;

    start_rtsp_streaming(8554, 5400, EncoderType::H264);

    Ok(bin)
}

fn start_rtsp_streaming(rtsp_port: u32, udpsink_port: u32, encoder: EncoderType) {
    let encoder_name = match encoder {
        EncoderType::H264 => "H264",
        EncoderType::H265 => "H265",
    };

    let udp_buffer_size: u64 = 512 * 1024;

    let udpsrc_pipeline = format!(
        "( udpsrc name=pay0 port={} buffer-size={} caps=\"application/x-rtp, media=video, clock-rate=90000, encoding-name={}, payload=96 \" )",
        udpsink_port, udp_buffer_size, encoder_name);

    let server = gst_rtsp_server::RTSPServer::new();
    server
        .set_property("service", rtsp_port.to_string())
        .unwrap();

    let mounts = server.mount_points().ok_or(NoMountPoints).unwrap();
    let factory = gst_rtsp_server::RTSPMediaFactory::new();
    factory.set_launch(udpsrc_pipeline.as_str());
    factory.set_shared(true);
    mounts.add_factory("/ds-video", &factory);

    let _id = server.attach(None).unwrap();

    info!(
        "Stream ready at rtsp://127.0.0.1:{}/ds-video",
        server.bound_port()
    );

    unsafe {
        RTSP_SERVERS.push(server);
    }
}
