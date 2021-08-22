use gst::prelude::*;

use anyhow::Error;

mod common;
use common::MissingElement;
use common::ErrorMessage;

pub mod sources;

pub struct Pipeline {
    pipeline: gst::Pipeline,
    sink: gst::Element,
}

impl Pipeline {
    pub fn new(display: bool) -> Result<Self, Error> {
        gst::init()?;

        let pipeline = gst::Pipeline::new(None);

        let sink;
        if display {
            sink = gst::ElementFactory::make("nveglglessink", None).map_err(|_| MissingElement("nveglglessink"))?;
        } else {
            sink = gst::ElementFactory::make("fakesink", None).map_err(|_| MissingElement("fakesink"))?;
        }

        pipeline.add_many(&[&sink])?;

        Ok(Pipeline{
            pipeline,
            sink
        })
    }

    pub fn add_source(&self, src: &dyn sources::Source) -> Result<(), Error> {
        self.pipeline.add_many(&[src.get_bin()])?;
        src.link(&self.sink)?;

        Ok(())
    }

    pub fn run(&self) -> Result<(), Error> {
        self.pipeline.set_state(gst::State::Playing)?;

        let bus = self.pipeline
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
