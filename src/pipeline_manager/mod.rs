use super::pipeline;
use super::pipeline::config::{PipelineConfig, SourceConfig};
use super::pipeline::Pipeline;

use anyhow::Error;
use log::error;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

pub struct PipelineManager {
    pipeline: Arc<Pipeline>,
    sources_config_hash: HashMap<u8, u64>,
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

        let mut manager = PipelineManager {
            pipeline: Arc::new(pipeline),
            sources_config_hash: HashMap::new(),
        };

        // add sources
        for src_config in pipeline_config.sources {
            manager.add_or_update_source(&src_config)?;
        }

        Ok(manager)
    }

    pub fn add_or_update_source(&mut self, config: &SourceConfig) -> Result<(), Error> {
        let source_id = config.id;

        if self.sources_config_hash.get(&source_id).is_some() {
            // if source alredy exist, remove it
            self.pipeline.remove_source()?;
        }

        // add source
        match &config.kind {
            pipeline::config::SourceKind::Test => {
                let src = pipeline::sources::TestSource::new().expect("Cant cerate test source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
            pipeline::config::SourceKind::Uri { uri } => {
                let src = pipeline::sources::URISource::new(uri.to_string())
                    .expect("Cant cerate uri source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
            pipeline::config::SourceKind::Rtsp { uri } => {
                let src = pipeline::sources::RTSPSource::new(uri.to_string())
                    .expect("Cant create rtsp source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
        };

        self.sources_config_hash.insert(source_id, config.get_hash());

        Ok(())
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
