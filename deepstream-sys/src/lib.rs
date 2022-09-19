use libc::c_int;
use gst_sys::GstEvent;

extern  "C" {
    pub fn gst_nvevent_new_stream_reset(source_id: c_int) -> *mut GstEvent;
}
