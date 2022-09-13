use anyhow::Error;

pub trait Source {
    fn get_bin(&self) -> &gst::Bin;
    fn link(&self, dst: &gst::Element) -> Result<(), Error>;
    fn on_remove(&mut self);
}
