use gst::prelude::*;
use anyhow::Error;

#[path = "./common.rs"]
mod common;
use common::MissingElement;


fn add_bin_ghostpad(bin: &gst::Bin, name: &str, pad: &gst::Pad) -> Result<(), Error> {
    //use glib::translate::*;
    //use gstreamer_sys::GstBin;
    //use gstreamer_sys::GstElement;
    //use gstreamer_sys::gst_ghost_pad_new;
    //use gstreamer_sys::gst_pad_set_active;
    //use gstreamer_sys::gst_element_add_pad;
    /*
        unsafe {
            let bin_mut :*mut GstBin = bin.to_glib_none().0;

            let ghost_pad_mut = gst_ghost_pad_new("sink".as_ptr() as *const i8, pad.to_glib_none().0);
            gst_pad_set_active (ghost_pad_mut, 1);
            gst_element_add_pad(&mut (*bin_mut).element, ghost_pad_mut);
            //(*bin_mut).element.add_pad(&ghost_pad_mut);
        }
    */
    let ghost_pad = gst::GhostPad::with_target(Some(name), pad).unwrap();
    bin.add_pad(&ghost_pad)?;

    Ok(())
}

pub struct TestSource {
    pub bin: gst::Bin,
}

pub trait Source {
    fn get_bin(&self) -> &gst::Bin;
    fn link(&self, dst: &gst::Element) -> Result<(), Error>;
}

impl TestSource {
    pub fn new() -> Result<Self, Error> {
        let bin = gst::Bin::new(Some("testsrcbin"));

        let src = gst::ElementFactory::make("videotestsrc", None)
            .map_err(|_| MissingElement("videotestsrc"))?;

        bin.add_many(&[&src])?;

        let pad = src.static_pad("src").expect("videotestsrc has no srcpad");
        add_bin_ghostpad(&bin, "sink", &pad)?;
        
        drop(pad);
    
        Ok(TestSource{
            bin
        })
    }
}

impl Source for TestSource {
    fn link(&self, dst: &gst::Element) -> Result<(), Error> {
        self.bin.link(dst).expect("Failed to link TestSourve");

        Ok(())
    }

    fn get_bin(&self) -> &gst::Bin {
        &self.bin
    }
}

pub struct URISource {
    bin: gst::Bin,
}

impl URISource {
    pub fn new(uri: String) -> Result<Self, Error> {
        let bin = gst::Bin::new(Some("urisourcebin"));

        let urisrc = gst::ElementFactory::make("uridecodebin", None)
            .map_err(|_| MissingElement("uridecodebin"))?;
        let queue = gst::ElementFactory::make("queue", None)
            .map_err(|_| MissingElement("queue"))?;
        
        // Config urisourcebin
        urisrc.set_property("uri", &uri.to_string())?;

        // Add elements to queue
        bin.add_many(&[&urisrc, &queue])?;

        // Add bin sink ghostpad
        let pad = queue.static_pad("src").expect("queue has no srcpad");
        add_bin_ghostpad(&bin, "sink", &pad)?;

        // Connect the pad-added signal
        let queue_weak = queue.downgrade();
        urisrc.connect_pad_added(move |src, src_pad| {
            println!("Received new pad {} from {}", src_pad.name(), src.name());
            
            let queue = match queue_weak.upgrade() {
                Some(queue) => queue,
                None => return,
            };

            let sink_pad = queue
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }
            let new_pad_caps = match src_pad.current_caps() {
                Some(cap) => cap,
                None => src_pad.query_caps(None),
            };
            let new_pad_struct = new_pad_caps
                .structure(0)
                .expect("Failed to get first structure of caps.");
            let new_pad_type = new_pad_struct.name();
    
            println!("Received pad type {}", new_pad_type);
            
            let is_video = new_pad_type.starts_with("video/x-raw");
            if !is_video {
                println!(
                    "It has type {} which is not video. Ignoring.",
                    new_pad_type
                );
                return;
            }

            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        });
    
        Ok(URISource{
            bin
        })
    }
}

impl Source for URISource {
    fn link(&self, dst: &gst::Element) -> Result<(), Error> {
        self.bin.link(dst)?;

        Ok(())
    }

    fn get_bin(&self) -> &gst::Bin {
        &self.bin
    }
}
