use gst::glib;
use gst::subclass::prelude::ObjectSubclass;
use gst_base::subclass::prelude::{BaseTransformImpl, ElementImpl, ObjectImpl};
//use gst::{gst_debug, gst_error, gst_info};

use once_cell::sync::Lazy;

use deepstream::gst_meta::{DsMeta, GstNvDsMetaType};

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "nvobjconv",
        gst::DebugColorFlags::empty(),
        Some("Transforms buffer objects to meta"),
    )
});

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
                println!("BatchNeta: {:?}", meta.batch_meta());
            }
        }
        Ok(gst::FlowSuccess::Ok)
    }
}
