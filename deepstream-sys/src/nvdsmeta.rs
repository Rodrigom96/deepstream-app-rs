#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong,
    c_ushort, c_void, intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

use glib_sys::{gpointer, GList, GRecMutex};

const MAX_USER_FIELDS: usize = 4;
const MAX_RESERVED_FIELDS: usize = 4;

pub type NvDsFrameMetaList = GList;
pub type NvDsUserMetaList = GList;
pub type NvDsMetaList = GList;

pub type NvDsMetaCopyFunc = gpointer;
pub type NvDsMetaReleaseFunc = gpointer;

pub type NvDsMetaType = c_int;
pub const NVDS_INVALID_META: NvDsMetaType = -1;
pub const NVDS_BATCH_META: NvDsMetaType = 1;
pub const NVDS_FRAME_META: NvDsMetaType = 2;
pub const NVDS_OBJ_META: NvDsMetaType = 3;
pub const NVDS_DISPLAY_META: NvDsMetaType = 4;
pub const NVDS_CLASSIFIER_META: NvDsMetaType = 5;
pub const NVDS_LABEL_INFO_META: NvDsMetaType = 6;
pub const NVDS_USER_META: NvDsMetaType = 7;
pub const NVDS_PAYLOAD_META: NvDsMetaType = 8;
pub const NVDS_EVENT_MSG_META: NvDsMetaType = 9;
pub const NVDS_OPTICAL_FLOW_META: NvDsMetaType = 10;
pub const NVDS_LATENCY_MEASUREMENT_META: NvDsMetaType = 11;
pub const NVDSINFER_TENSOR_OUTPUT_META: NvDsMetaType = 12;
pub const NVDSINFER_SEGMENTATION_META: NvDsMetaType = 13;
pub const NVDS_RESERVED_META: NvDsMetaType = 4095;
pub const NVDS_GST_CUSTOM_META: NvDsMetaType = 4096;
pub const NVDS_START_USER_META: NvDsMetaType = NVDS_GST_CUSTOM_META + 4096 + 1;
pub const NVDS_FORCE32_META: NvDsMetaType = 0x7FFFFFFF;

#[repr(C)]
pub struct NvDsMetaPool {
    pub meta_type: NvDsMetaType,
    pub max_elements_in_pool: c_uint,
    pub element_size: c_uint,
    pub num_empty_elements: c_uint,
    pub num_full_elements: c_uint,
    pub empty_list: *mut NvDsMetaList,
    pub full_list: *mut NvDsMetaList,
    pub copy_func: NvDsMetaCopyFunc,
    pub release_func: NvDsMetaReleaseFunc,
}

#[repr(C)]
pub struct NvDsBaseMeta {
    pub batch_meta: *mut NvDsBatchMeta,
    pub meta_type: NvDsMetaType,
    pub uContext: *mut c_void,
    pub copy_func: NvDsMetaCopyFunc,
    pub release_func: NvDsMetaReleaseFunc,
}

#[repr(C)]
pub struct NvDsBatchMeta {
    pub base_meta: NvDsBaseMeta,
    pub max_frames_in_batch: c_uint,
    pub num_frames_in_batch: c_uint,
    pub frame_meta_pool: *mut NvDsMetaPool,
    pub obj_meta_pool: *mut NvDsMetaPool,
    pub display_meta_pool: *mut NvDsMetaPool,
    pub user_meta_pool: *mut NvDsMetaPool,
    pub label_info_meta_pool: *mut NvDsMetaPool,
    pub frame_meta_list: *mut NvDsFrameMetaList,
    pub batch_user_meta_list: *mut NvDsUserMetaList,
    pub meta_mutex: GRecMutex,
    pub misc_batch_info: [c_longlong; MAX_USER_FIELDS],
    pub reserved: [c_longlong; MAX_RESERVED_FIELDS],
}
