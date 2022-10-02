use gst_sys::GstEvent;
use libc::c_int;

pub mod gst_nvdsmeta;
pub mod nvds_roi_meta;
pub mod nvdsmeta;
pub mod nvdsmeta_schema;
pub mod nvll_osd_struct;

extern "C" {
    pub fn gst_nvevent_new_stream_reset(source_id: c_int) -> *mut GstEvent;
}
