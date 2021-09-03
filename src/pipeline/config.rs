use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SourceKind {
    Test,
    Uri { uri: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    pub id: u8,
    pub kind: SourceKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterConfig {
    NvInfer { config_path: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgBrokerSinkConfig {
    pub topic: String,
    pub server: String,
    pub port: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SinksConfig {
    pub display: bool,
    pub msg_broker: MsgBrokerSinkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub sources: Vec<SourceConfig>,
    pub filters: Vec<FilterConfig>,
    pub sinks: SinksConfig,
}

impl PipelineConfig {
    pub fn from_file(filename: &str) -> Result<Self, serde_yaml::Error> {
        let f = std::fs::File::open(filename).unwrap();
        let config: PipelineConfig = serde_yaml::from_reader(f)?;

        Ok(config)
    }
}
