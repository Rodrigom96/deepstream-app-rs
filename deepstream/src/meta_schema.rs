use std::ffi::{CStr, CString};

use deepstream_sys::nvdsmeta_schema as ffi;

#[repr(transparent)]
pub struct NvDsRect(ffi::NvDsRect);

impl NvDsRect {
    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsRect) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn new(top: f32, left: f32, width: f32, height: f32) -> Self {
        Self(ffi::NvDsRect {
            top,
            left,
            width,
            height,
        })
    }

    pub fn top(&self) -> f32 {
        self.0.top
    }

    pub fn left(&self) -> f32 {
        self.0.left
    }

    pub fn width(&self) -> f32 {
        self.0.width
    }

    pub fn height(&self) -> f32 {
        self.0.height
    }
}

impl From<NvDsRect> for ffi::NvDsRect {
    fn from(r: NvDsRect) -> Self {
        r.0
    }
}

impl std::fmt::Debug for NvDsRect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsRect")
            .field("top", &self.0.top)
            .field("left", &self.0.left)
            .field("width", &self.0.width)
            .field("height", &self.0.height)
            .finish()
    }
}

#[repr(transparent)]
pub struct NvDsEventMsgMeta(ffi::NvDsEventMsgMeta);

impl NvDsEventMsgMeta {
    pub fn as_mut_ptr(&self) -> *mut ffi::NvDsEventMsgMeta {
        self as *const Self as *mut ffi::NvDsEventMsgMeta
    }

    pub unsafe fn from_ptr<'a>(ptr: *mut ffi::NvDsEventMsgMeta) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn new(
        bbox: NvDsRect,
        obj_class_id: i32,
        obj_class_label: String,
        sensor_id: i32,
        frame_id: i32,
        confidence: f64,
        tracking_id: i32,
        ts: String,
    ) -> Self {
        let label_c_chars = CString::new(obj_class_label).unwrap();
        let ts_c_chars = CString::new(ts).unwrap();

        Self(ffi::NvDsEventMsgMeta {
            bbox: ffi::NvDsRect::from(bbox),
            obj_class_id,
            obj_class_label: label_c_chars.into_raw(),
            sensor_id,
            frame_id,
            confidence,
            tracking_id,
            ts: ts_c_chars.into_raw(),
        })
    }

    pub fn bbox<'a>(&self) -> &'a NvDsRect {
        unsafe {
            let ptr = &self.0.bbox as *const ffi::NvDsRect as *mut ffi::NvDsRect;
            NvDsRect::from_ptr::<'a>(ptr)
        }
    }

    pub fn obj_class_label<'a>(&self) -> &'a str {
        unsafe {
            CStr::from_ptr::<'a>(self.0.obj_class_label)
                .to_str()
                .unwrap()
        }
    }

    pub fn ts<'a>(&self) -> &'a str {
        unsafe { CStr::from_ptr::<'a>(self.0.ts).to_str().unwrap() }
    }
}

impl std::fmt::Debug for NvDsEventMsgMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsEventMsgMeta")
            .field("bbox", &self.bbox())
            .field("obj_class_label", &self.obj_class_label())
            .field("frame_id", &self.0.frame_id)
            .field("ts", &self.ts())
            .finish()
    }
}
