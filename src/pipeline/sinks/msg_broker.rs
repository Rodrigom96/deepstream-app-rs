use anyhow::Error;
use gst::prelude::*;
use log::{warn};

use super::super::common;
use super::super::config::MsgBrokerSinkConfig;
use common::MissingElement;

/// Return a bin with nveglglessink
pub fn create_bin(name: Option<&str>, config: MsgBrokerSinkConfig) -> Result<gst::Bin, Error> {
    let bin = gst::Bin::new(name);

    let queue = gst::ElementFactory::make("queue", None).map_err(|_| MissingElement("queue"))?;
    let obj_transform =
        gst::ElementFactory::make("nvobjconv", None).map_err(|_| MissingElement("nvobjconv"))?;
    let transform =
        gst::ElementFactory::make("nvmsgconv", None).map_err(|_| MissingElement("nvmsgconv"))?;
    let sink = gst::ElementFactory::make("nvmsgbroker", None)
        .map_err(|_| MissingElement("nvmsgbroker"))?;

    // set threshold on queue to avoid pipeline choke when broker is stuck on network
    // * leaky=2 (2): downstream       - Leaky on downstream (old buffers)
    queue.set_property_from_str("leaky", "downstream");
    queue.set_property("max-size-buffers", 2_u32)?;
    queue.connect("overrun", false, move |_args| {
        warn!("nvmsgbroker queue overrun; Older Message Buffer");
        None
    })?;
    sink.set_property("proto-lib", "/opt/nvidia/deepstream/deepstream-6.0/lib/libnvds_kafka_proto.so")?;
    sink.set_property("conn-str", format!("{};{}", config.server, config.port))?;
    sink.set_property("topic", config.topic)?;
    sink.set_property("config", "config/filters/msgbroker_config.txt")?;
    sink.set_property("sync", false)?;

    bin.add_many(&[&queue, &obj_transform, &transform, &sink])?;
    common::add_bin_ghost_pad(&bin, &queue, "sink")?;
    queue.link(&obj_transform)?;
    obj_transform.link(&transform)?;
    transform.link(&sink)?;

    Ok(bin)
}
