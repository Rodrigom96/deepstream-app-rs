use anyhow::Error;
use gst::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ds::gst_meta::{DsMeta, GstNvDsMetaType};

struct Metric {
    last_instant: Option<Instant>,
    fps_vec: Vec<f64>,
    n: usize,
    max_seconds: u64,
}

impl Metric {
    fn new() -> Self {
        Self {
            last_instant: None,
            fps_vec: Vec::new(),
            n: 100,
            max_seconds: 60,
        }
    }

    fn update(&mut self) {
        // update fps list
        if let Some(last_instant) = self.last_instant {
            // reset
            if last_instant.elapsed() > Duration::new(self.max_seconds, 0) {}

            self.fps_vec
                .push(1.0 / last_instant.elapsed().as_secs_f64());
            if self.fps_vec.len() > self.n {
                self.fps_vec.remove(0);
            }
        }

        // update instant
        self.last_instant = Some(Instant::now());
    }

    fn fps(&self) -> Option<f64> {
        if self.fps_vec.len() == 0 {
            return None;
        }

        if let Some(last_instant) = self.last_instant {
            if last_instant.elapsed() > Duration::new(self.max_seconds, 0) {
                return None;
            }
        }

        // return average fps
        let mut sum = 0.0;
        for fps in &self.fps_vec {
            sum += fps
        }
        Some(sum / self.fps_vec.len() as f64)
    }
}

pub struct FPSMetrics {
    metric_by_source: Arc<Mutex<HashMap<u8, Metric>>>,
}

impl FPSMetrics {
    pub fn new(bin: &gst::Bin) -> Result<Self, Error> {
        let fps_metrics = Self {
            metric_by_source: Arc::new(Mutex::new(HashMap::new())),
        };

        let srcpad: gst::Pad = bin.static_pad("src").expect("Failed to get srcpad");
        let metric_by_source_clone = fps_metrics.metric_by_source.clone();
        srcpad.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if let Some(gst::PadProbeData::Buffer(ref mut buffer)) = &mut info.data {
                let buffer = buffer.make_mut();

                let mut metrics = metric_by_source_clone.lock().unwrap();

                for mut meta in buffer.iter_meta_mut::<DsMeta>() {
                    if let GstNvDsMetaType::BatchGstMeta = meta.meta_type() {
                        let mut batch_meta = meta.batch_meta().unwrap();
                        for mut frame in batch_meta.iter_frame() {
                            // get frame metric
                            let source_id = frame.source_id();
                            let mut metric =
                                metrics.entry(source_id as u8).or_insert(Metric::new());

                            // update metric
                            metric.update();
                        }
                    }
                }
            }

            gst::PadProbeReturn::Ok
        });

        Ok(fps_metrics)
    }

    pub fn fps(&self, source_id: &u8) -> Option<f64> {
        let metric_by_source: std::sync::MutexGuard<HashMap<u8, Metric>> =
            self.metric_by_source.lock().unwrap();

        if let Some(metric) = metric_by_source.get(source_id) {
            metric.fps()
        } else {
            None
        }
    }
}
