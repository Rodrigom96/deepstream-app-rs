use gst::prelude::*;

use anyhow::Error;
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<String>,
    source: glib::error::Error,
}

pub struct Pipeline {
    pipeline: gst::Pipeline,
    //src: gst::Element,
    //sink: gst::Element,
}

impl Pipeline {
    pub fn new(display: bool) -> Result<Self, Error> {
        gst::init()?;

        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None)
            .map_err(|_| MissingElement("videotestsrc"))?;

        let sink;
        if display {
            sink = gst::ElementFactory::make("autovideosink", None).map_err(|_| MissingElement("autovideosink"))?;
        } else {
            sink = gst::ElementFactory::make("fakesink", None).map_err(|_| MissingElement("fakesink"))?;
        }

        pipeline.add_many(&[&src, &sink])?;
        src.link(&sink)?;

        Ok(Pipeline{
            pipeline
        })
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
