use libc::c_void;
use std::marker::PhantomData;

use deepstream_sys::nvdsmeta as ffi;

#[repr(transparent)]
pub struct NvDsObjectMeta(ffi::NvDsObjectMeta);

impl NvDsObjectMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsObjectMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn class_id(&self) -> i32 {
        self.0.class_id
    }
}

impl std::fmt::Debug for NvDsObjectMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsObjectMeta")
            .field("class_id", &self.class_id())
            .finish()
    }
}

pub struct NvDsObjectMetaIter<'a> {
    ptr: Option<std::ptr::NonNull<ffi::NvDsObjectMetaList>>,
    phantom: PhantomData<&'a NvDsObjectMeta>,
}

impl<'a> NvDsObjectMetaIter<'a> {
    pub fn new(list: *mut ffi::NvDsObjectMetaList) -> Self {
        let ptr = std::ptr::NonNull::new(list);
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for NvDsObjectMetaIter<'a> {
    type Item = &'a mut NvDsObjectMeta;

    fn next(&mut self) -> Option<&'a mut NvDsObjectMeta> {
        match self.ptr {
            None => None,
            Some(cur) => unsafe {
                self.ptr = std::ptr::NonNull::new(cur.as_ref().next);

                let mut item = &mut *(cur.as_ref().data as *mut NvDsObjectMeta);

                Some(item)
            },
        }
    }
}

#[repr(transparent)]
pub struct NvDsFrameMeta(ffi::NvDsFrameMeta);

impl NvDsFrameMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsFrameMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn frame_num(&self) -> i32 {
        self.0.frame_num
    }

    pub fn source_id(&self) -> u32 {
        self.0.source_id
    }

    pub fn iter_objects<'a>(&mut self) -> NvDsObjectMetaIter<'a> {
        NvDsObjectMetaIter::new(self.0.obj_meta_list)
    }
}

impl std::fmt::Debug for NvDsFrameMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsFrameMeta")
            .field("frame_num", &self.frame_num())
            .field("source_id", &self.source_id())
            .finish()
    }
}

pub struct NvDsFrameMetaIter<'a> {
    ptr: Option<std::ptr::NonNull<ffi::NvDsFrameMetaList>>,
    phantom: PhantomData<&'a NvDsFrameMeta>,
}

impl<'a> NvDsFrameMetaIter<'a> {
    pub fn new(list: *mut ffi::NvDsFrameMetaList) -> Self {
        let ptr = std::ptr::NonNull::new(list);
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for NvDsFrameMetaIter<'a> {
    type Item = &'a mut NvDsFrameMeta;

    fn next(&mut self) -> Option<&'a mut NvDsFrameMeta> {
        match self.ptr {
            None => None,
            Some(cur) => unsafe {
                self.ptr = std::ptr::NonNull::new(cur.as_ref().next);

                let mut item = &mut *(cur.as_ref().data as *mut NvDsFrameMeta);

                Some(item)
            },
        }
    }
}

#[repr(transparent)]
pub struct NvDsBatchMeta(ffi::NvDsBatchMeta);

impl NvDsBatchMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut c_void) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn iter_frame<'a>(&mut self) -> NvDsFrameMetaIter<'a> {
        NvDsFrameMetaIter::new(self.0.frame_meta_list)
    }
}

impl std::fmt::Debug for NvDsBatchMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsBatchMeta")
            .field("meta_type", &self.0.base_meta.meta_type)
            .finish()
    }
}
