use anyhow::Error;
use gst::prelude::*;

use crate::common::SourceId;

use super::common;
use super::config::SinksConfig;
use common::MissingElement;

mod msg_broker;
mod render_sink;
mod rtsp_sink;

pub struct PipelineSink {
    pub bin: gst::Bin,
    rtsp_demux: Option<rtsp_sink::RTSPDemuxSink>,
}

impl PipelineSink {
    pub fn new(config: SinksConfig) -> Result<Self, Error> {
        rtsp_sink::init(8554);

        let bin = gst::Bin::new(Some("sink_bin"));

        let queue =
            gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
        let nvvidconv = gst::ElementFactory::make("nvvideoconvert", None)
            .map_err(|_| MissingElement("nvvideoconvert"))?;
        let nvosd =
            gst::ElementFactory::make("nvdsosd", None).map_err(|_| MissingElement("nvdsosd"))?;
        let tee = gst::ElementFactory::make("tee", None).map_err(|_| MissingElement("tee"))?;
        bin.add_many(&[&queue, &nvvidconv, &nvosd, &tee])?;
        queue.link(&nvvidconv)?;
        nvvidconv.link(&nvosd)?;
        nvosd.link(&tee)?;

        // Add kafka msg broker
        if let Some(broker_config) = config.msg_broker {
            let broker = msg_broker::create_bin(Some("kafkamsgbroker_sink"), broker_config)?;
            bin.add(&broker)?;
            common::link_element_to_tee_src_pad(&tee, &broker)?;
        }

        // Add rtsp demuxer
        let rtsp_demux: Option<rtsp_sink::RTSPDemuxSink> = match config.rtsp {
            true => {
                let rtsp_demux = rtsp_sink::RTSPDemuxSink::new(Some("rtsp_demux"))?;
                bin.add(&rtsp_demux.bin)?;
                common::link_element_to_tee_src_pad(&tee, &rtsp_demux.bin)?;
                Some(rtsp_demux)
            }
            false => None,
        };

        // Add display sinks
        if config.display {
            let render_sink = render_sink::create_bin(Some("render_sink"))?;
            bin.add(&render_sink)?;
            common::link_element_to_tee_src_pad(&tee, &render_sink)?;
        }

        common::add_bin_ghost_pad(&bin, &queue, "sink")?;

        Ok(PipelineSink { bin, rtsp_demux })
    }

    pub fn add_source_sink(&self, id: &SourceId) -> Result<(), Error> {
        if let Some(rtsp_demux) = &self.rtsp_demux {
            rtsp_demux.add_sink(id)?;
        }

        Ok(())
    }

    pub fn remove_source_sink(&self, id: &SourceId) -> Result<(), Error> {
        if let Some(rtsp_demux) = &self.rtsp_demux {
            rtsp_demux.remove_sink(id)?;
        }

        Ok(())
    }
}
