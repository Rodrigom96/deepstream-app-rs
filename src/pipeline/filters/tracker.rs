use anyhow::Error;
use gst::prelude::*;

use super::super::common;
use common::MissingElement;

pub fn create_element(
    lib_path: Option<String>,
    config_path: Option<String>,
) -> Result<gst::Element, Error> {
    let tracker =
        gst::ElementFactory::make("nvtracker", None).map_err(|_| MissingElement("nvtracker"))?;

    tracker.set_property("tracker-width", 640u32)?;
    tracker.set_property("tracker-height", 384u32)?;

    match lib_path {
        Some(ref path) => tracker.set_property("ll-lib-file", path)?,
        _ => tracker.set_property(
            "ll-lib-file",
            "/opt/nvidia/deepstream/deepstream-6.0/lib/libnvds_nvmultiobjecttracker.so",
        )?,
    };
    match config_path {
        Some(ref path) => tracker.set_property("ll-config-file", path)?,
        _ => tracker.set_property(
            "ll-config-file",
            "config/filters/config_tracker_NvDCF_perf.yml",
        )?,
    };
    tracker.set_property("enable-batch-process", true)?;
    Ok(tracker)
}
