use gst::prelude::*;
use gst::MessageView;

use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::thread;

use log::{debug, error};

mod common;
use common::MissingElement;

pub mod config;
mod filters;
mod metrics;
mod sinks;
pub mod sources;

pub struct Pipeline {
    pipeline: gst::Pipeline,
    streammux: gst::Element,
    pipeline_sink: sinks::PipelineSink,
    sources_bin_name: HashMap<u8, String>,
    fps_metrics: metrics::FPSMetrics,
}

impl Pipeline {
    pub fn new(
        filters_config: Vec<config::FilterConfig>,
        sinks_config: config::SinksConfig,
    ) -> Result<Self, Error> {
        gst::init()?;

        let pipeline = gst::Pipeline::new(None);

        // create elementes
        let streammux = create_streamux().expect("Cant create steamux");
        let filters_bin = filters::create_bin(filters_config)?;
        let pipeline_sink = sinks::PipelineSink::new(sinks_config)?;
        // add elements
        pipeline.add_many(&[&streammux])?;
        pipeline.add(&filters_bin)?;
        pipeline.add(&pipeline_sink.bin)?;

        // link elements
        streammux
            .link(&filters_bin)
            .expect("Failed to link streamux with filters_bin");
        filters_bin
            .link(&pipeline_sink.bin)
            .expect("Failed to link filters_bin with sink");

        let fps_metrics = metrics::FPSMetrics::new(&filters_bin)?;

        Ok(Pipeline {
            pipeline,
            streammux,
            pipeline_sink,
            sources_bin_name: HashMap::new(),
            fps_metrics,
        })
    }

    pub fn add_source(&mut self, src: &dyn sources::Source, id: &u8) -> Result<(), Error> {
        debug!("Adding source {} ...", id);
        if self.sources_bin_name.get(id).is_some() {
            return Err(anyhow!("Source {} alredy in pipelein", id));
        }

        let bin = src.get_bin();
        self.pipeline.add_many(&[bin])?;
        let sink_name = format!("sink_{}", id);

        let sinkpad = self
            .streammux
            .request_pad_simple(&sink_name)
            .expect("Cant get streamux sinkpad");
        let srcpad = bin.static_pad("src").expect("Catn get source bin srcpad");
        srcpad.link(&sinkpad)?;

        // Start bin
        bin.sync_state_with_parent()?;

        // Add source rtsp sink
        self.pipeline_sink.add_source_sink(id)?;

        self.sources_bin_name.insert(*id, bin.name().to_string());

        debug!("Source {} added with name {}", id, bin.name());
        Ok(())
    }

    pub fn remove_source(&mut self, id: &u8) -> Result<(), Error> {
        debug!("Removing source {} ...", id);
        if let Some(bin_name) = self.sources_bin_name.remove(id) {
            // get source bin
            let bin = self.pipeline.by_name(&bin_name).unwrap();

            // stop bin
            bin.set_state(gst::State::Null)?;

            // unlink source bin from streamux
            let sink_name = format!("sink_{}", id);
            let sinkpad = self
                .streammux
                .static_pad(&sink_name)
                .expect("Cant get streamux sinkpad");
            sinkpad.send_event(gst::event::FlushStop::new(false));
            self.streammux.release_request_pad(&sinkpad);

            self.pipeline.remove(&bin)?;

            // Remove source sink
            self.pipeline_sink.remove_source_sink(id)?;

            debug!("Source {} removed with name {}", id, bin_name);
        } else {
            return Err(anyhow!("Source {} not found", id));
        }

        Ok(())
    }

    pub fn start(&self) -> Result<(), Error> {
        let main_loop = glib::MainLoop::new(None, false);

        let bus = self
            .pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");
        bus.add_signal_watch();

        let pipeline_weak = self.pipeline.downgrade();
        let main_loop_clone = main_loop.clone();
        bus.connect_message(None, move |_, msg| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };

            match msg.view() {
                // Just set the pipeline to READY (which stops playback).
                MessageView::Eos(..) => {
                    pipeline
                        .set_state(gst::State::Ready)
                        .expect("Unable to set the pipeline to the `Ready` state");
                }
                MessageView::Error(err) => {
                    let main_loop = &main_loop_clone;
                    error!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    main_loop.quit();
                    // stop pipeline on error
                    pipeline.set_state(gst::State::Null).unwrap();
                }
                _ => (),
            }
        });

        self.pipeline.set_state(gst::State::Playing)?;

        thread::spawn(move || main_loop.run());
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let (_, pipeline_state, _) = self.pipeline.state(None);
        pipeline_state != gst::State::Null
    }

    pub fn sources_fps(&self) -> HashMap<u8, Option<f64>> {
        let mut sources_fps = HashMap::new();
        for source_id in self.sources_bin_name.keys() {
            sources_fps.insert(*source_id, self.fps_metrics.fps(source_id));
        }

        sources_fps
    }
}

/// Create nvstreammux element and config it.
fn create_streamux() -> Result<gst::Element, Error> {
    let streammux = gst::ElementFactory::make("nvstreammux", None)
        .map_err(|_| MissingElement("nvstreammux"))?;

    // Set propertys
    streammux.set_property("batch-size", 1_u32)?;
    streammux.set_property("enable-padding", true)?;
    streammux.set_property("live-source", true)?;
    streammux.set_property("width", 1280_u32)?;
    streammux.set_property("height", 720_u32)?;

    Ok(streammux)
}
