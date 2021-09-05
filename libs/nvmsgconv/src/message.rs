use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::str;

use super::nvdsmeta_custom_schema::NvDsEvent;

#[derive(Serialize, Deserialize)]
struct Object {
    id: i32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    label: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    frame_id: u64,
    timestamp: String,
    camera_id: u32,
    objects: Vec<Object>,
}

pub fn generate_message(events: &[NvDsEvent]) -> String {
    let mut objects = Vec::new();

    // parse events objects
    for event in events {
        let meta = unsafe { &*event.metadata };
        
        // get label String
        let c_str: &CStr = unsafe { CStr::from_ptr(meta.obj_class_label) };
        let str_slice: &str = c_str.to_str().unwrap();
        let label: String = str_slice.to_owned();
        
        let obj = Object {
            id: meta.tracking_id,
            x: meta.bbox.left as u32,
            y: meta.bbox.top as u32,
            width: meta.bbox.width as u32,
            height: meta.bbox.height as u32,
            label,
        };

        objects.push(obj);
    }

    // get first event meta
    let meta = unsafe { &*events[0].metadata };
    
    // get timestamp String
    let c_str: &CStr = unsafe { CStr::from_ptr(meta.ts) };
    let str_slice: &str = c_str.to_str().unwrap();
    let timestamp: String = str_slice.to_owned();
    
    let message = Message {
        frame_id: meta.frame_id as u64,
        timestamp,
        camera_id: meta.sensor_id as u32,
        objects,
    };

    // serealize message to json
    serde_json::to_string(&message).unwrap()
}
