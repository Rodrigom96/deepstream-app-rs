use libc::c_void;

use deepstream_sys::nvdsmeta as ffi;

#[repr(transparent)]
pub struct NvDsBatchMeta(ffi::NvDsBatchMeta);

impl NvDsBatchMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut c_void) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }
}

impl std::fmt::Debug for NvDsBatchMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsBatchMeta")
            .field("meta_type", &self.0.base_meta.meta_type)
            .finish()
    }
}
