#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

#[allow(unused_imports)]
use glib_sys::{gboolean, gconstpointer, gpointer, GType};

#[repr(C)]
pub enum NvDsEventType {
    Entry,
    Exit,
    Moving,
    Stopped,
    Empty,
    Parked,
    Reset,
}

#[repr(C)]
pub enum NvDsPayloadType {
    Deepstream,
    DeepstreamMinimal,
}

#[repr(C)]
pub struct NvDsRect {
    pub top: c_float,
    pub left: c_float,
    pub width: c_float,
    pub height: c_float,
}

#[repr(C)]
pub struct NvDsEventMsgMeta {
    pub bbox: NvDsRect,
    pub obj_class_id: c_int,
    pub obj_class_label: *mut c_char,
    pub sensor_id: c_int,
    pub frame_id: c_int,
    pub confidence: c_double,
    pub tracking_id: c_int,
    pub ts: *mut c_char,
}

#[repr(C)]
pub struct NvDsEvent {
    pub event_type: NvDsEventType,
    pub metadata: *mut NvDsEventMsgMeta,
}

#[repr(C)]
pub struct NvDsPayload {
    pub payload: gpointer,
    pub payload_size: c_uint,
    pub component_id: c_uint,
}
