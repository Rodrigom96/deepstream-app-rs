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
        let c_str: &CStr = unsafe { CStr::from_ptr(meta.objClassLabel) };
        let str_slice: &str = c_str.to_str().unwrap();
        let label: String = str_slice.to_owned();
        
        let obj = Object {
            id: meta.trackingId,
            x: meta.bbox.left as u32,
            y: meta.bbox.top as u32,
            width: meta.bbox.width as u32,
            height: meta.bbox.height as u32,
            label: label,
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
        frame_id: meta.frameId as u64,
        timestamp: timestamp,
        camera_id: meta.sensorId as u32,
        objects: objects,
    };

    // serealize message to json
    let serialized = serde_json::to_string(&message).unwrap();
    serialized
}
