use deepstream_sys::nvdsmeta_schema as ffi;

#[repr(transparent)]
pub struct NvDsRect(ffi::NvDsRect);

impl NvDsRect {
    pub fn new(top: f32, left: f32, width: f32, height: f32) -> Self {
        Self(ffi::NvDsRect {
            top,
            left,
            width,
            height,
        })
    }
}

impl From<NvDsRect> for ffi::NvDsRect {
    fn from(r: NvDsRect) -> Self {
        r.0
    }
}

#[repr(transparent)]
pub struct NvDsEventMsgMeta(ffi::NvDsEventMsgMeta);

impl NvDsEventMsgMeta {
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
        let label_bytes = obj_class_label.into_bytes();
        let mut label_c_chars: Vec<i8> = label_bytes.iter().map(|c| *c as i8).collect::<Vec<i8>>();
        label_c_chars.push(0); // null terminator

        let ts_bytes = ts.into_bytes();
        let mut ts_c_chars: Vec<i8> = ts_bytes.iter().map(|c| *c as i8).collect::<Vec<i8>>();
        ts_c_chars.push(0); // null terminator

        Self(ffi::NvDsEventMsgMeta {
            bbox: ffi::NvDsRect::from(bbox),
            obj_class_id,
            obj_class_label: label_c_chars.as_mut_ptr(),
            sensor_id,
            frame_id,
            confidence,
            tracking_id,
            ts: ts_c_chars.as_mut_ptr(),
        })
    }
}

impl std::fmt::Debug for NvDsEventMsgMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NvDsEventMsgMeta")
            .field("obj_class_id", &self.0.obj_class_id)
            .field("frame_id", &self.0.frame_id)
            .finish()
    }
}
