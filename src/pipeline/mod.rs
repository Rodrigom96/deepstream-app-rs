use gst::prelude::*;

use anyhow::Error;

mod common;
use common::{ErrorMessage, MissingElement};

pub mod config;
mod filters;
mod sinks;
pub mod sources;

pub struct Pipeline {
    pipeline: gst::Pipeline,
    streammux: gst::Element,
}

impl Pipeline {
    pub fn new(display: bool, filters_config: Vec<config::FilterConfig>) -> Result<Self, Error> {
        gst::init()?;

        let pipeline = gst::Pipeline::new(None);

        // create elementes
        let streammux = create_streamux().expect("Cant create steamux");
        let filters_bin = filters::create_bin(filters_config)?;
        let sink = sinks::create_sink_bin(display).expect("Cant create sink_bin");
        // add elements
        pipeline.add_many(&[&streammux])?;
        pipeline.add(&filters_bin)?;
        pipeline.add(&sink)?;

        // link elements
        streammux
            .link(&filters_bin)
            .expect("Failed to link streamux with filters_bin");
        filters_bin
            .link(&sink)
            .expect("Failed to link streamux with filters_bin");

        Ok(Pipeline {
            pipeline,
            streammux,
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

        Ok(())
    }

    pub fn run(&self) -> Result<(), Error> {
        self.pipeline.set_state(gst::State::Playing)?;

        let bus = self
            .pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    self.pipeline.set_state(gst::State::Null)?;
                    return Err(ErrorMessage {
                        src: msg
                            .src()
                            .map(|s| String::from(s.path_string()))
                            .unwrap_or_else(|| String::from("None")),
                        error: err.error().to_string(),
                        debug: err.debug(),
                        source: err.error(),
                    }
                    .into());
                }
                _ => (),
            }
        }

        self.pipeline.set_state(gst::State::Null)?;

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
