use super::common::add_bin_ghost_pad;
use super::config::FilterConfig;
use anyhow::Error;
use gst::prelude::*;

mod nvinfer;

pub fn create_bin(filters_config: Vec<FilterConfig>) -> Result<gst::Bin, Error> {
    let num_filters = filters_config.len();
    assert!(num_filters > 0);

    let bin = gst::Bin::new(Some("filter_bin"));

    let mut last_element: Option<gst::Element> = None;
    for (i, config) in filters_config.iter().enumerate() {
        // create element
        let elem = match config {
            FilterConfig::NvInfer { config_path } => {
                nvinfer::create_element(config_path.to_string())?
            }
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
