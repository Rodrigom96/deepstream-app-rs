use glib::translate::{from_glib, FromGlib, IntoGlib};
use gstreamer::meta::MetaAPI;

use deepstream_sys::gst_nvdsmeta as ffi;

use crate::meta::NvDsBatchMeta;

#[derive(Debug)]
pub enum GstNvDsMetaType {
    #[doc(alias = "NVDS_GST_INVALID_META")]
    Invalid,
    #[doc(alias = "NVDS_BATCH_GST_META")]
    BatchGstMeta,
    #[doc(alias = "NVDS_DECODER_GST_META")]
    DecoderGstMeta,
    #[doc(alias = "NVDS_DEWARPER_GST_META")]
    DewarperGstMeta,
    #[doc(alias = "NVDS_RESERVED_GST_META")]
    ReservedGstMeta,
    #[doc(alias = "NVDS_GST_META_FORCE32")]
    GstMetaForce32,
    #[doc(hidden)]
    __Unknown(i32),
}

#[doc(hidden)]
impl IntoGlib for GstNvDsMetaType {
    type GlibType = ffi::GstNvDsMetaType;

    fn into_glib(self) -> ffi::GstNvDsMetaType {
        match self {
            Self::Invalid => ffi::NVDS_GST_INVALID_META,
            Self::BatchGstMeta => ffi::NVDS_BATCH_GST_META,
            Self::DecoderGstMeta => ffi::NVDS_DECODER_GST_META,
            Self::DewarperGstMeta => ffi::NVDS_DEWARPER_GST_META,
            Self::ReservedGstMeta => ffi::NVDS_RESERVED_GST_META,
            Self::GstMetaForce32 => ffi::NVDS_GST_META_FORCE32,
            Self::__Unknown(value) => value,
        }
    }
}

#[doc(hidden)]
impl FromGlib<ffi::GstNvDsMetaType> for GstNvDsMetaType {
    unsafe fn from_glib(val: ffi::GstNvDsMetaType) -> Self {
        match val {
            ffi::NVDS_GST_INVALID_META => Self::Invalid,
            ffi::NVDS_BATCH_GST_META => Self::BatchGstMeta,
            ffi::NVDS_DECODER_GST_META => Self::DecoderGstMeta,
            ffi::NVDS_DEWARPER_GST_META => Self::DewarperGstMeta,
            ffi::NVDS_RESERVED_GST_META => Self::ReservedGstMeta,
            ffi::NVDS_GST_META_FORCE32 => Self::GstMetaForce32,
            _ => Self::__Unknown(val),
        }
    }
}

#[repr(transparent)]
pub struct DsMeta(ffi::NvDsMeta);

impl DsMeta {
    pub fn meta_type(&self) -> GstNvDsMetaType {
        unsafe { from_glib(self.0.meta_type) }
    }

    pub fn batch_meta(&mut self) -> Option<&mut NvDsBatchMeta> {
        if let GstNvDsMetaType::BatchGstMeta = self.meta_type() {
            unsafe { Some(NvDsBatchMeta::from_ptr(self.0.meta_data)) }
        } else {
            None
        }
    }
}

unsafe impl Send for DsMeta {}
unsafe impl Sync for DsMeta {}

unsafe impl MetaAPI for DsMeta {
    type GstType = ffi::NvDsMeta;

    #[doc(alias = "nvds_meta_api_get_type")]
    fn meta_api() -> gstreamer::glib::Type {
        unsafe { from_glib(ffi::nvds_meta_api_get_type()) }
    }
}

impl std::fmt::Debug for DsMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DsMeta")
            .field("meta_type", &self.meta_type())
            .finish()
    }
}
