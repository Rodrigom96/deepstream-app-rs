use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
pub struct MissingElement(#[error(not(source))] pub &'static str);

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
pub struct ErrorMessage {
    pub src: String,
    pub error: String,
    pub debug: Option<String>,
    pub source: glib::error::Error,
}
