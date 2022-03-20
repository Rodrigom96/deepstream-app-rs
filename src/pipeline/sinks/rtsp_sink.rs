use anyhow::Error;
use derive_more::{Display, Error};
use gst::prelude::*;
use gst_rtsp_server::prelude::*;
use log::info;
use state::LocalStorage;

use super::super::common;
use common::MissingElement;

enum EncoderType {
    H264,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Could not get mount points")]
struct NoMountPoints;

static mut SERVER: LocalStorage<gst_rtsp_server::RTSPServer> = LocalStorage::new();

pub fn init(rtsp_port: u32) {
    unsafe {
        SERVER.set(gst_rtsp_server::RTSPServer::new);

        let server = SERVER.get();
        server
            .set_property("service", rtsp_port.to_string())
            .unwrap();
        let _id = server.attach(None).unwrap();
    }
}

pub fn create_bin(name: Option<&str>, rtsp_path: &str, udp_port: u32) -> Result<gst::Bin, Error> {
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
    sink.set_property("port", udp_port as i32)?;
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

    start_rtsp_streaming(rtsp_path, udp_port, EncoderType::H264);

    Ok(bin)
}

fn start_rtsp_streaming(rtsp_path: &str, udpsink_port: u32, encoder: EncoderType) {
    let encoder_name = match encoder {
        EncoderType::H264 => "H264",
    };

    let udp_buffer_size: u64 = 512 * 1024;

    let udpsrc_pipeline = format!(
        "( udpsrc name=pay0 port={} buffer-size={} caps=\"application/x-rtp, media=video, clock-rate=90000, encoding-name={}, payload=96 \" )",
        udpsink_port, udp_buffer_size, encoder_name);

    unsafe {
        let server = SERVER.get();
        let mounts = server.mount_points().ok_or(NoMountPoints).unwrap();
        let factory = gst_rtsp_server::RTSPMediaFactory::new();
        factory.set_launch(udpsrc_pipeline.as_str());
        factory.set_shared(true);
        mounts.add_factory(&format!("/{}", rtsp_path), &factory);
        info!(
            "Stream ready at rtsp://127.0.0.1:{}/{}",
            server.bound_port(),
            rtsp_path
        );
    }
}

pub struct RTSPDemuxSink {
    pub bin: gst::Bin,
    streamdemux: gst::Element,
}

impl RTSPDemuxSink {
    pub fn new(name: Option<&str>) -> Result<Self, Error> {
        let bin = gst::Bin::new(name);

        let streamdemux = gst::ElementFactory::make("nvstreamdemux", None)
            .map_err(|_| MissingElement("nvstreamdemux"))?;

        bin.add_many(&[&streamdemux])?;
        common::add_bin_ghost_pad(&bin, &streamdemux, "sink")?;

        Ok(RTSPDemuxSink { bin, streamdemux })
    }

    pub fn add_sink(&self, id: &u8) -> Result<(), Error> {
        let src_name = format!("src_{}", id);

        let sink = create_bin(
            Some(&format!("rtspbin_{}", id)),
            &format!("cam/{}", id),
            5401 + (*id as u32),
        )?;
        self.bin.add(&sink)?;

        let srcpad = self
            .streamdemux
            .request_pad_simple(&src_name[..])
            .expect("Cant get streamdemux srcpad");
        let sinkpad = sink.static_pad("sink").expect("Catn get rtsp bin sinkpad");
        srcpad.link(&sinkpad)?;

        Ok(())
    }
}
