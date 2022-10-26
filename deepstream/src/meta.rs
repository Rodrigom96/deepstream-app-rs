use glib::ffi::gpointer;
use libc::{c_char, c_void};
use std::ffi::CStr;
use std::marker::PhantomData;

use deepstream_sys::nvds_roi_meta::NvOSD_RectParams;
use deepstream_sys::nvdsmeta as ffi;

use crate::meta_schema::NvDsEventMsgMeta;

#[repr(transparent)]
pub struct NvDsObjectMeta(ffi::NvDsObjectMeta);

impl NvDsObjectMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsObjectMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn class_id(&self) -> i32 {
        self.0.class_id
    }

    pub fn obj_label(&self) -> &str {
        unsafe {
            CStr::from_ptr(&self.0.obj_label as *const c_char)
                .to_str()
                .unwrap()
        }
    }

    pub fn object_id(&self) -> u64 {
        self.0.object_id
    }

    pub fn rect_params(&self) -> &NvOSD_RectParams {
        &self.0.rect_params
    }

    pub fn confidence(&self) -> f32 {
        self.0.confidence
    }
}

impl std::fmt::Debug for NvDsObjectMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsObjectMeta")
            .field("class_id", &self.class_id())
            .field("obj_label", &self.obj_label())
            .field("object_id", &self.object_id())
            .field("confidence", &self.confidence())
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
    pub fn as_mut_ptr(&self) -> *mut ffi::NvDsFrameMeta {
        self as *const Self as *mut ffi::NvDsFrameMeta
    }

    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsFrameMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn frame_number(&self) -> i32 {
        self.0.frame_num
    }

    pub fn source_id(&self) -> u32 {
        self.0.source_id
    }

    pub fn iter_objects<'a>(&mut self) -> NvDsObjectMetaIter<'a> {
        NvDsObjectMetaIter::new(self.0.obj_meta_list)
    }

    #[doc(alias = "nvds_add_user_meta_to_frame")]
    pub fn add_user_meta<T>(&mut self, user_meta: &NvDsUserMeta<T>) {
        unsafe {
            let frame_ptr = self.as_mut_ptr();
            let user_meta_ptr = user_meta.as_mut_ptr();
            ffi::nvds_add_user_meta_to_frame(frame_ptr, user_meta_ptr);
        }
    }
}

impl std::fmt::Debug for NvDsFrameMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsFrameMeta")
            .field("frame_number", &self.frame_number())
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
pub struct NvDsBaseMeta(ffi::NvDsBaseMeta);

impl NvDsBaseMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsBaseMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }
}

impl std::fmt::Debug for NvDsBaseMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsBaseMeta")
            .field("meta_type", &self.0.meta_type)
            .finish()
    }
}

#[repr(transparent)]
pub struct NvDsUserMeta<T>(ffi::NvDsUserMeta, PhantomData<T>);

impl<T> NvDsUserMeta<T> {
    pub fn as_mut_ptr(&self) -> *mut ffi::NvDsUserMeta {
        self as *const Self as *mut ffi::NvDsUserMeta
    }

    pub fn base_meta<'a>(&self) -> &'a NvDsBaseMeta {
        unsafe {
            let ptr = &self.0.base_meta as *const ffi::NvDsBaseMeta as *mut ffi::NvDsBaseMeta;
            NvDsBaseMeta::from_ptr::<'a>(ptr)
        }
    }
}

unsafe extern "C" fn msg_copy_func(data: gpointer, user_data: gpointer) -> gpointer {
    println!("--- copy event msg ---");

    let user_meta = data as *mut ffi::NvDsUserMeta;
    let src_meta = (*user_meta).user_meta_data as *const NvDsEventMsgMeta as *mut NvDsEventMsgMeta;
    let dst_meta = std::mem::ManuallyDrop::new(src_meta.clone());
    Box::into_raw(Box::new(dst_meta)) as gpointer
}

unsafe extern "C" fn msg_release_func(data: gpointer, user_data: gpointer) {
    let user_meta = data as *mut ffi::NvDsUserMeta;
    let src_meta = (*user_meta).user_meta_data as *mut NvDsEventMsgMeta;
    src_meta.drop_in_place();
}

impl NvDsUserMeta<NvDsEventMsgMeta> {
    pub fn set_data(&mut self, data: NvDsEventMsgMeta) {
        let user_meta_data = std::mem::ManuallyDrop::new(data);
        self.0.user_meta_data = Box::into_raw(Box::new(user_meta_data)) as gpointer;
        self.0.base_meta.meta_type = ffi::NVDS_EVENT_MSG_META;
        self.0.base_meta.copy_func = Some(msg_copy_func);
        self.0.base_meta.release_func = Some(msg_release_func);
    }
}

impl<T> std::fmt::Debug for NvDsUserMeta<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsUserMeta")
            .field("base_meta", self.base_meta())
            .finish()
    }
}

#[repr(transparent)]
pub struct NvDsBatchMeta(ffi::NvDsBatchMeta);

impl NvDsBatchMeta {
    pub unsafe fn from_ptr<'a>(ptr: *mut c_void) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn as_mut_ptr(&self) -> *mut ffi::NvDsBatchMeta {
        self as *const Self as *mut ffi::NvDsBatchMeta
    }

    pub fn base_meta<'a>(&self) -> &'a NvDsBaseMeta {
        unsafe {
            let ptr = &self.0.base_meta as *const ffi::NvDsBaseMeta as *mut ffi::NvDsBaseMeta;
            NvDsBaseMeta::from_ptr::<'a>(ptr)
        }
    }

    pub fn iter_frame<'a>(&mut self) -> NvDsFrameMetaIter<'a> {
        NvDsFrameMetaIter::new(self.0.frame_meta_list)
    }

    #[doc(alias = "nvds_acquire_user_meta_from_pool")]
    pub fn acquire_user_meta<T>(&mut self) -> &mut NvDsUserMeta<T> {
        unsafe {
            let ptr = self.as_mut_ptr();
            let user_meta_ptr = ffi::nvds_acquire_user_meta_from_pool(ptr) as *mut NvDsUserMeta<T>;

            &mut *user_meta_ptr
        }
    }
}

impl std::fmt::Debug for NvDsBatchMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsBatchMeta")
            .field("base_meta", self.base_meta())
            .finish()
    }
}
