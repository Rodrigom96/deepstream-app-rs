use gst::prelude::*;
use gst::MessageView;

use anyhow::Error;

use log::error;

mod common;
use common::MissingElement;

pub mod config;
mod filters;
mod sinks;
pub mod sources;

pub struct Pipeline {
    pipeline: gst::Pipeline,
    streammux: gst::Element,
    pipeline_sink: sinks::PipelineSink,
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

        Ok(Pipeline {
            pipeline,
            streammux,
            pipeline_sink,
        })
    }

    pub fn add_source(&self, src: &dyn sources::Source, id: u8) -> Result<(), Error> {
        let bin = src.get_bin();
        self.pipeline.add_many(&[bin])?;
        let sink_name = format!("sink_{}", id);

        let sinkpad = self
            .streammux
            .request_pad_simple(&sink_name[..])
            .expect("Cant get streamux sinkpad");
        let srcpad = bin.static_pad("src").expect("Catn get source bin srcpad");
        srcpad.link(&sinkpad)?;

        // Add source rtsp sink
        self.pipeline_sink.add_source_sink(id)?;

        Ok(())
    }

    pub fn remove_source(&self) -> Result<(), Error> {
        panic!("Remove source not implemented")
    }

    pub fn run(&self) -> Result<(), Error> {
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
                }
                _ => (),
            }
        });

        self.pipeline.set_state(gst::State::Playing)?;

        main_loop.run();

        self.pipeline.set_state(gst::State::Null)?;

        bus.remove_signal_watch();

        Ok(())
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
