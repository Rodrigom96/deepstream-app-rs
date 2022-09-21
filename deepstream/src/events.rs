use gstreamer::Event;

/// Creates a "custom reset" event for the specified source.
/// # Arguments
///
/// * `source_id` - Source ID of the stream for which reset is to be sent; also the pad ID of the sinkpad of the Gst-nvstreammux plugin for which the source is configured.
pub fn new_stream_reset(source_id: i32) -> Option<Event> {
    unsafe {
        let ptr = deepstream_sys::gst_nvevent_new_stream_reset(source_id);
        if ptr.is_null() {
            None
        } else {
            Some(Event::from_glib_full(ptr))
        }
    }
}
