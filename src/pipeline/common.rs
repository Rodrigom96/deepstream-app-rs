use anyhow::Error;
use derive_more::{Display, Error};

use gst::prelude::*;

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

pub fn add_bin_ghost_pad_named(
    bin: &gst::Bin,
    elem: &gst::Element,
    pad_name: &str,
    ghost_pad_name: &str,
) -> Result<(), Error> {
    let pad = match elem.static_pad(pad_name) {
        Some(pad) => pad,
        None => {
            panic!("Could not find {} pad", pad_name);
        }
    };
    let ghost_pad = gst::GhostPad::with_target(Some(ghost_pad_name), &pad).unwrap();
    bin.add_pad(&ghost_pad)?;

    Ok(())
}

pub fn add_bin_ghost_pad(bin: &gst::Bin, elem: &gst::Element, pad_name: &str) -> Result<(), Error> {
    add_bin_ghost_pad_named(bin, elem, pad_name, pad_name)
}

/// Link sink pad of a element to source pad of tee.
pub fn link_element_to_tee_src_pad<P: IsA<gst::Element>>(
    tee: &gst::Element,
    elem: &P,
) -> Result<(), Error> {
    let pad_template = tee
        .pad_template("src_%u")
        .expect("Cant get tee src pad template");
    let tee_src_pad = tee
        .request_pad(&pad_template, None, None)
        .expect("Can request tee pad");

    let sink_pad = elem.static_pad("sink").expect("Cant get sink pad");

    tee_src_pad.link(&sink_pad)?;

    Ok(())
}
