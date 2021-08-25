use anyhow::Error;
use gst::prelude::*;

use super::super::common;
use common::MissingElement;

pub fn create_element(config_path: String) -> Result<gst::Element, Error> {
    let pgie = gst::ElementFactory::make("nvinfer", None)
        .map_err(|_| MissingElement("nvinfer"))?;

    pgie.set_property("config-file-path", config_path)?;

    Ok(pgie)
}
