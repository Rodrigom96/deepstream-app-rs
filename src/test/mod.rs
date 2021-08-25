use super::pipeline::config::PipelineConfig;

#[test]
fn load_pipeline_config() {
    let config = PipelineConfig::from_file("config/pipeline_config.yml").unwrap();
    assert_eq!(config.sources[0].id, 1);
}
