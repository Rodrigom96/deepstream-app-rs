#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong,
    c_ushort, c_void, intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

#[repr(C)]
pub struct NvBbox_Coords {
    pub left: c_float,
    pub top: c_float,
    pub width: c_float,
    pub height: c_float,
}

#[repr(C)]
pub struct NvOSD_ColorParams {
    pub red: c_double,
    pub green: c_double,
    pub blue: c_double,
    pub alpha: c_double,
}
