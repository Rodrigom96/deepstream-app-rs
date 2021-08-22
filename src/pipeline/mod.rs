use gst::prelude::*;

use anyhow::Error;

mod common;
use common::{ErrorMessage, MissingElement};

pub mod sources;

pub struct Pipeline {
    pipeline: gst::Pipeline,
    streammux: gst::Element,
}

impl Pipeline {
    pub fn new(display: bool) -> Result<Self, Error> {
        gst::init()?;

        let pipeline = gst::Pipeline::new(None);

        let streammux = create_streamux().expect("Cant create steamux");

        let sink;
        if display {
            sink = add_display_sink(&pipeline).unwrap();
        } else {
            sink = gst::ElementFactory::make("fakesink", None)
                .map_err(|_| MissingElement("fakesink"))?;
            pipeline.add(&sink)?;
        }

        pipeline.add_many(&[&streammux])?;
        streammux.link(&sink)?;

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

fn create_streamux() -> Result<gst::Element, Error> {
    let streammux = gst::ElementFactory::make("nvstreammux", None)
        .map_err(|_| MissingElement("nvstreammux"))?;

    // Set propiertys
    streammux.set_property("batch-size", 1 as u32)?;
    streammux.set_property("enable-padding", true)?;
    streammux.set_property("live-source", true)?;
    streammux.set_property("width", 1280 as u32)?;
    streammux.set_property("height", 720 as u32)?;

    Ok(streammux)
}

fn add_display_sink(pipeline: &gst::Pipeline) -> Result<gst::Element, Error> {
    let nvvidconv = gst::ElementFactory::make("nvvideoconvert", None)
        .map_err(|_| MissingElement("nvvideoconvert"))?;
    let tiler = gst::ElementFactory::make("nvmultistreamtiler", None)
        .map_err(|_| MissingElement("nvmultistreamtiler"))?;
    let sink = gst::ElementFactory::make("nveglglessink", None)
        .map_err(|_| MissingElement("nveglglessink"))?;

    pipeline.add_many(&[&nvvidconv, &tiler, &sink])?;

    nvvidconv.link(&tiler)?;
    tiler.link(&sink)?;

    Ok(nvvidconv)
}
