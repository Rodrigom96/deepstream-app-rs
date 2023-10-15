use anyhow::Error;
use gst::prelude::*;

use super::common::{add_bin_ghost_pad, MissingElement};
use super::config::FilterConfig;

mod nvinfer;
mod tracker;

pub fn create_bin(filters_config: Vec<FilterConfig>) -> Result<gst::Bin, Error> {
    let num_filters = filters_config.len();

    let bin = gst::Bin::new(Some("filter_bin"));

    if num_filters == 0 {
        let queue =
            gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
        bin.add(&queue)?;
        add_bin_ghost_pad(&bin, &queue, "sink")?;
        add_bin_ghost_pad(&bin, &queue, "src")?;

        return Ok(bin);
    }

    let mut last_element: Option<gst::Element> = None;
    for (i, config) in filters_config.iter().enumerate() {
        // create element
        let elem = match config {
            FilterConfig::NvInfer { config_path } => {
                nvinfer::create_element(config_path.to_string())?
            }
            FilterConfig::Tracker {
                lib_path,
                config_path,
            } => tracker::create_element(lib_path.clone(), config_path.clone())?,
        };

        // add element to bin
        bin.add(&elem)?;

        // link element
        if i == 0 {
            add_bin_ghost_pad(&bin, &elem, "sink")?;
        } else if let Some(last_elem) = last_element {
            last_elem.link(&elem)?
        }
        if i == num_filters - 1 {
            add_bin_ghost_pad(&bin, &elem, "src")?;
        }

        last_element = Some(elem);
    }

    Ok(bin)
}
