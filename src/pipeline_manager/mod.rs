use super::pipeline;
use super::pipeline::config::PipelineConfig;
use super::pipeline::Pipeline;

use anyhow::Error;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

pub struct PipelineManager {
    pipeline: Arc<Pipeline>,
}

impl PipelineManager {
    pub fn new(filename: &str) -> Result<Self, Error> {
        let pipeline_config = match PipelineConfig::from_file(filename) {
            Ok(pipeline_config) => pipeline_config,
            Err(e) => panic!("Error load pipelin config file, {}", e),
        };

        let pipeline = match Pipeline::new(pipeline_config.filters, pipeline_config.sinks) {
            Ok(r) => r,
            Err(e) => {
                panic!("Error! {}", e);
            }
        };

        load_sources(&pipeline, pipeline_config.sources);

        Ok(PipelineManager {
            pipeline: Arc::new(pipeline),
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        // run pipeline in background
        let pipeline = self.pipeline.clone();
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();
        thread::spawn(move || match pipeline.run() {
            Ok(_) => {
                thread_running.store(false, Ordering::Relaxed);
            }
            Err(e) => {
                error!("Error in pipeline running, {}", e);
                thread_running.store(false, Ordering::Relaxed);
            }
        });

        let delay = time::Duration::from_secs(10);
        loop {
            thread::sleep(delay);

            // stop loop if pipeline not running
            if !running.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }
}

fn load_sources(pipe: &pipeline::Pipeline, sources_config: Vec<pipeline::config::SourceConfig>) {
    for src_config in sources_config {
        let id = src_config.id;

        // create src and add to pipeline
        match src_config.kind {
            pipeline::config::SourceKind::Test => {
                let src = pipeline::sources::TestSource::new().expect("Cant cerate test source");
                pipe.add_source(&src, id).expect("Cant add source");
            }
            pipeline::config::SourceKind::Uri { uri } => {
                let src = pipeline::sources::URISource::new(uri).expect("Cant cerate uri source");
                pipe.add_source(&src, id).expect("Cant add source");
            }
            pipeline::config::SourceKind::Rtsp { uri } => {
                let src = pipeline::sources::RTSPSource::new(uri).expect("Cant create rtsp source");
                pipe.add_source(&src, id).expect("Cant add source");
            }
        };
    }
}
