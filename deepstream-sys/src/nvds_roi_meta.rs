#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong,
    c_ushort, c_void, intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

use crate::nvll_osd_struct;

#[repr(C)]
pub struct NvOSD_FontParams {
    pub font_name: *mut c_char,
    pub font_size: c_int,
    pub font_color: nvll_osd_struct::NvOSD_ColorParams,
}

#[repr(C)]
pub struct NvOSD_TextParams {
    pub display_text: *mut c_char,
    pub x_offset: c_uint,
    pub y_offset: c_uint,
    pub font_params: NvOSD_FontParams,
    pub set_bg_clr: c_int,
    pub text_bg_clr: nvll_osd_struct::NvOSD_ColorParams,
}

#[repr(C)]
pub struct NvOSD_RectParams {
    pub left: c_float,
    pub top: c_float,
    pub width: c_float,
    pub height: c_float,
    pub border_width: c_uint,
    pub border_color: nvll_osd_struct::NvOSD_ColorParams,
    pub has_bg_color: c_uint,
    pub reserved: c_uint,
    pub bg_color: nvll_osd_struct::NvOSD_ColorParams,
    pub has_color_info: c_int,
    pub color_id: c_int,
}

#[repr(C)]
pub struct NvOSD_MaskParams {
    pub data: *mut c_float,
    pub size: c_uint,
    pub threshold: c_float,
    pub width: c_int,
    pub height: c_int,
}
