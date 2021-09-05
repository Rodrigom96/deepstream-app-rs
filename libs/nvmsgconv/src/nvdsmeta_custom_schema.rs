use glib_sys as glib;

#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

#[allow(unused_imports)]
use glib::{gboolean, gconstpointer, gpointer, GType};

#[repr(C)]
pub enum NvDsEventType {
    NVDS_EVENT_ENTRY,
    NVDS_EVENT_EXIT,
    NVDS_EVENT_MOVING,
    NVDS_EVENT_STOPPED,
    NVDS_EVENT_EMPTY,
    NVDS_EVENT_PARKED,
    NVDS_EVENT_RESET,
}

#[repr(C)]
pub enum NvDsPayloadType {
    NVDS_PAYLOAD_DEEPSTREAM,
    NVDS_PAYLOAD_DEEPSTREAM_MINIMAL,
    NVDS_PAYLOAD_RESERVED,
    NVDS_PAYLOAD_CUSTOM,
    NVDS_PAYLOAD_FORCE32,
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
    pub objClassId: c_int,
    pub objClassLabel: *mut c_char,
    pub sensorId: c_int,
    pub frameId: c_int,
    pub confidence: c_double,
    pub trackingId: c_int,
    pub ts: *mut c_char,
}

#[repr(C)]
pub struct NvDsEvent {
    pub eventType: NvDsEventType,
    pub metadata: *mut NvDsEventMsgMeta,
}

#[repr(C)]
pub struct NvDsPayload {
    pub payload: gpointer,
    pub payloadSize: c_uint,
    pub componentId: c_uint,
}
