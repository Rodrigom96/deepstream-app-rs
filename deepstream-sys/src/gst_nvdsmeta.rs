#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

use glib_sys::{gpointer, GType};
use gst_sys::GstMeta;

use crate::nvdsmeta;

extern  "C" {
    pub fn nvds_meta_api_get_type() -> GType;
}

pub type GstNvDsMetaType = c_int;
pub const NVDS_GST_INVALID_META: GstNvDsMetaType = -1;
pub const NVDS_BATCH_GST_META: GstNvDsMetaType = nvdsmeta::NVDS_GST_CUSTOM_META + 1;
pub const NVDS_DECODER_GST_META: GstNvDsMetaType = NVDS_BATCH_GST_META + 1;
pub const NVDS_DEWARPER_GST_META: GstNvDsMetaType = NVDS_DECODER_GST_META + 1;
pub const NVDS_RESERVED_GST_META: GstNvDsMetaType = nvdsmeta::NVDS_GST_CUSTOM_META + 4096;
pub const NVDS_GST_META_FORCE32: GstNvDsMetaType = 0x7FFFFFFF;

pub type NvDsMetaCopyFunc = gpointer;
pub type NvDsMetaReleaseFunc = gpointer;

#[repr(C)]
pub struct NvDsMeta {
    pub meta: GstMeta,
    pub meta_data: gpointer,
    pub user_data: gpointer,
    pub meta_type: c_int,
    pub copyfunc: NvDsMetaCopyFunc,
    pub freefunc: NvDsMetaReleaseFunc,
    pub gst_to_nvds_meta_transform_func: NvDsMetaCopyFunc,
    pub gst_to_nvds_meta_release_func: NvDsMetaReleaseFunc,
}
