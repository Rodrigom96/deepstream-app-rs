#[allow(unused_imports)]
use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
    intptr_t, size_t, ssize_t, time_t, uintptr_t, FILE,
};

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
pub const NVDS_RESERVED_META: NvDsMetaType =  4095;
pub const NVDS_GST_CUSTOM_META: NvDsMetaType =  4096;
pub const NVDS_START_USER_META: NvDsMetaType =  NVDS_GST_CUSTOM_META + 4096 + 1;
pub const NVDS_FORCE32_META: NvDsMetaType =  0x7FFFFFFF;
