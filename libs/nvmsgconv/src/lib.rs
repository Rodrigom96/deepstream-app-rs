#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};
use std::ffi::CString;

mod nvdsmeta_custom_schema;
use nvdsmeta_custom_schema::*;
mod message;

pub struct NvDsMsg2pCtx {}

#[no_mangle]
pub extern "C" fn nvds_msg2p_ctx_create(
    _file: *const c_char,
    _payload_type: NvDsPayloadType,
) -> *mut NvDsMsg2pCtx {
    let ctx = NvDsMsg2pCtx {};

    Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_ctx_destroy(ctx: *mut NvDsMsg2pCtx) {
    drop(ctx);
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_generate(
    _ctx: *mut NvDsMsg2pCtx,
    events: *const NvDsEvent,
    size: c_uint,
) -> *mut NvDsPayload {
    let payload = generate_payload(events, size);
    Box::into_raw(Box::new(payload))
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_generate_multiple(
    _ctx: *mut NvDsMsg2pCtx,
    events: *const NvDsEvent,
    size: c_uint,
    payload_count: *mut c_uint,
) -> *mut *mut NvDsPayload {
    let mut payloads = Vec::new();

    let payload = generate_payload(events, size);
    payloads.push(Box::into_raw(Box::new(payload)));
    unsafe {
        *payload_count += 1;
    }

    payloads.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_generate_new(
    _ctx: *mut NvDsMsg2pCtx,
    events: *const NvDsEvent,
    size: c_uint,
) -> *mut NvDsPayload {
    panic!("Not implemented");
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_generate_multiple_new(
    _ctx: *mut NvDsMsg2pCtx,
    events: *const NvDsEvent,
    size: c_uint,
    payload_count: *mut c_uint,
) -> *mut *mut NvDsPayload {
    panic!("Not implemented");
}

#[no_mangle]
pub extern "C" fn nvds_msg2p_release(_ctx: *mut NvDsMsg2pCtx, payload: *mut NvDsPayload) {
    unsafe {
        drop((*payload).payload);
    }
    drop(payload);
}

fn generate_payload(events: *const NvDsEvent, size: c_uint) -> NvDsPayload {
    let events_vec =
        unsafe { std::slice::from_raw_parts(events as *const NvDsEvent, size as usize) };

    let message_str = message::generate_message(events_vec);
    let message_len = message_str.len();
    let c_str = CString::new(message_str).unwrap();
    let c_payload_message = c_str.into_raw();

    NvDsPayload {
        payload: c_payload_message as *mut c_void,
        payload_size: message_len as u32,
        component_id: 0,
    }
}
