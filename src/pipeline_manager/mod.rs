use crate::common::SourceId;

use super::pipeline;
use super::pipeline::config::{PipelineConfig, SourceConfig};
use super::pipeline::Pipeline;

use anyhow::Error;
use log::debug;
use std::collections::HashMap;
use std::{thread, time};

pub struct PipelineManager {
    pipeline: Pipeline,
    config_filename: String,
    sources_config_hash: HashMap<SourceId, u64>,
}

impl PipelineManager {
    pub fn new(filename: &str) -> Result<Self, Error> {
        let pipeline_config = match PipelineConfig::from_file(filename) {
            Ok(pipeline_config) => pipeline_config,
            Err(e) => panic!("Error load pipelin config file, {}", e),
        };

        let pipeline = match Pipeline::new(
            pipeline_config.streammux,
            pipeline_config.filters,
            pipeline_config.sinks,
        ) {
            Ok(r) => r,
            Err(e) => {
                panic!("Error! {}", e);
            }
        };

        let mut manager = PipelineManager {
            pipeline,
            config_filename: filename.to_string(),
            sources_config_hash: HashMap::new(),
        };

        manager.update_config()?;

        Ok(manager)
    }

    pub fn add_or_update_source(&mut self, config: &SourceConfig) -> Result<(), Error> {
        let source_id = &config.id;

        if let Some(old_config_hash) = self.sources_config_hash.get(source_id) {
            // skip if same config
            if old_config_hash == &config.get_hash() {
                debug!("Same config of source {}, skip update", source_id);
                return Ok(());
            }

            // if source alredy exist, remove it
            self.pipeline.remove_source(source_id)?;
        }

        // add source
        match &config.kind {
            pipeline::config::SourceKind::Test => {
                let src = pipeline::sources::TestSource::new().expect("Cant cerate test source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
            pipeline::config::SourceKind::Uri {
                uri,
                username,
                password,
            } => {
                let src = pipeline::sources::URISource::new(
                    uri,
                    username.as_deref(),
                    password.as_deref(),
                )
                .expect("Cant cerate uri source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
            pipeline::config::SourceKind::Rtsp {
                uri,
                username,
                password,
            } => {
                let src = pipeline::sources::RTSPSource::new(
                    &uri,
                    username.as_deref(),
                    password.as_deref(),
                )
                .expect("Cant create rtsp source");
                self.pipeline
                    .add_source(&src, source_id)
                    .expect("Cant add source");
            }
        };

        self.sources_config_hash
            .insert(*source_id, config.get_hash());

        Ok(())
    }

    pub fn remove_source(&mut self, id: &SourceId) -> Result<(), Error> {
        if self.sources_config_hash.remove(id).is_some() {
            self.pipeline.remove_source(id)?;
        }

        Ok(())
    }

    pub fn update_config(&mut self) -> Result<(), Error> {
        // load config file
        let filename = &self.config_filename;
        let pipeline_config = match PipelineConfig::from_file(filename) {
            Ok(pipeline_config) => pipeline_config,
            Err(e) => panic!("Error load pipelin config file, {}", e),
        };

        // add or update sources
        for src_config in &pipeline_config.sources {
            self.add_or_update_source(src_config)?;
        }

        // delete sources not in config
        let mut src_id_to_remove = Vec::new();
        for src_id in self.sources_config_hash.keys() {
            let mut not_found = true;
            for src_config in &pipeline_config.sources {
                if src_id == &(src_config.id) {
                    not_found = false;
                    break;
                }
            }
            if not_found {
                src_id_to_remove.push(src_id.to_owned());
            }
        }
        for src_id in src_id_to_remove.iter() {
            self.remove_source(src_id)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        // start pipeline
        self.pipeline.start()?;

        let delay = time::Duration::from_secs(10);
        loop {
            thread::sleep(delay);

            if !self.pipeline.is_running() {
                break;
            }

            // log fps
            let fps = self.pipeline.sources_fps();
            log::debug!("FPS: {:?}", fps);

            // sync with config
            self.update_config()?;
        }

        Ok(())
    }
}
