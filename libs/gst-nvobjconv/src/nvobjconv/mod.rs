use gst::glib;
use gst::prelude::*;

mod imp;

glib::wrapper! {
    pub struct NVObjconv(ObjectSubclass<imp::NVObjconv>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

unsafe impl Send for NVObjconv {}
unsafe impl Sync for NVObjconv {}


pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "nvobjconv",
        gst::Rank::None,
        NVObjconv::static_type(),
    )
}
