use chrono::prelude::{DateTime, Utc};
use gst::glib;
use gst::subclass::prelude::ObjectSubclass;
use gst_base::subclass::prelude::{BaseTransformImpl, ElementImpl, ObjectImpl};

use once_cell::sync::Lazy;

use deepstream::gst_meta::{DsMeta, GstNvDsMetaType};
use deepstream::meta_schema::{NvDsEventMsgMeta, NvDsRect};

#[derive(Default)]
pub struct NVObjconv {}

impl NVObjconv {}

#[glib::object_subclass]
impl ObjectSubclass for NVObjconv {
    const NAME: &'static str = "NVObjconv";
    type Type = super::NVObjconv;
    type ParentType = gst_base::BaseTransform;
}

impl ObjectImpl for NVObjconv {}

impl ElementImpl for NVObjconv {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "NVObjects Converter",
                "Filter/Metadata",
                "Transforms buffer objects to meta",
                "deepstream-rs",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst::Caps::new_any();
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for NVObjconv {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_ip(
        &self,
        _element: &Self::Type,
        buf: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        for mut meta in buf.iter_meta_mut::<DsMeta>() {
            if let GstNvDsMetaType::BatchGstMeta = meta.meta_type() {
                let ts = Utc::now().to_rfc3339();

                let mut batch_meta = meta.batch_meta().unwrap();
                for mut frame in batch_meta.iter_frame() {
                    for mut obj in frame.iter_objects() {
                        let msg_meta = NvDsEventMsgMeta::new(
                            NvDsRect::new(
                                obj.rect_params().top,
                                obj.rect_params().left,
                                obj.rect_params().width,
                                obj.rect_params().height,
                            ),
                            obj.class_id(),
                            obj.obj_label(),
                            frame.source_id().try_into().unwrap(),
                            frame.frame_number(),
                            f64::from(obj.confidence()),
                            obj.object_id().try_into().unwrap(),
                            &ts,
                        );

                        let mut user_meta = batch_meta.acquire_user_meta::<NvDsEventMsgMeta>();

                        user_meta.set_data(msg_meta);

                        frame.add_user_meta(user_meta);
                    }
                }
            }
        }
        Ok(gst::FlowSuccess::Ok)
    }
}
