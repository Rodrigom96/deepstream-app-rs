use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum SourceKind {
    Test,
    Uri {
        uri: String,
        username: Option<String>,
        password: Option<String>,
    },
    Rtsp {
        uri: String,
        username: Option<String>,
        password: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SourceConfig {
    pub id: u8,
    pub kind: SourceKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterConfig {
    NvInfer {
        config_path: String,
    },
    Tracker {
        lib_path: Option<String>,
        config_path: Option<String>,
    },
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
    pub rtsp: bool,
    pub msg_broker: Option<MsgBrokerSinkConfig>,
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

impl SourceConfig {
    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}
