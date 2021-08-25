mod lib;
mod logging;
mod pipeline;

#[cfg(test)]
mod test;

fn main() {
    // Init logging
    logging::init();

    let config = lib::Config::new();
    let pipeline_config =
        match pipeline::config::PipelineConfig::from_file("config/pipeline_config.yml") {
            Ok(pipeline_config) => pipeline_config,
            Err(e) => panic!("Error load pipelin config file, {}", e),
        };

    let pipe;
    match pipeline::Pipeline::new(config.display, pipeline_config.filters) {
        Ok(r) => pipe = r,
        Err(e) => {
            panic!("Error! {}", e);
        }
    }

    load_sources(&pipe, pipeline_config.sources);

    match pipe.run() {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
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
        };
    }
}
