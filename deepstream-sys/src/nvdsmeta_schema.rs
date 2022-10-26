#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

#[allow(unused_imports)]
use glib_sys::{gboolean, gconstpointer, gpointer, GType};

pub type NvDsEventType = c_int;
pub const NVDS_EVENT_ENTRY: NvDsEventType = 0;
pub const NVDS_EVENT_EXIT: NvDsEventType = 1;
pub const NVDS_EVENT_MOVING: NvDsEventType = 2;
pub const NVDS_EVENT_STOPPED: NvDsEventType = 3;
pub const NVDS_EVENT_EMPTY: NvDsEventType = 4;
pub const NVDS_EVENT_PARKED: NvDsEventType = 5;
pub const NVDS_EVENT_RESET: NvDsEventType = 6;
pub const NVDS_EVENT_RESERVED: NvDsEventType = 0x100;
pub const NVDS_EVENT_CUSTOM: NvDsEventType = 0x101;
pub const NVDS_EVENT_FORCE32: NvDsEventType = 0x7FFFFFFF;

pub type NvDsPayloadType = c_int;
pub const NVDS_PAYLOAD_DEEPSTREAM: NvDsPayloadType = 0;
pub const NVDS_PAYLOAD_DEEPSTREAM_MINIMAL: NvDsPayloadType = 1;
pub const NVDS_PAYLOAD_RESERVED: NvDsPayloadType = 0x100;
pub const NVDS_PAYLOAD_CUSTOM: NvDsPayloadType = 0x101;
pub const NVDS_PAYLOAD_FORCE32: NvDsPayloadType = 0x7FFFFFFF;

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
