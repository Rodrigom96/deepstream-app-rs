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
    transform.set_property("config", "config/filters/msgconv_config.txt")?;
    transform.set_property("msg2p-lib", config.lib)?;
    transform.set_property_from_str("payload-type", "NVDS_PAYLOAD_DEEPSTREAM_MINIMAL");
    sink.set_property("proto-lib", "/opt/nvidia/deepstream/deepstream-5.1/lib/libnvds_kafka_proto.so")?;
    sink.set_property("conn-str", format!("{};{}", config.server, config.port))?;
    sink.set_property("topic", config.topic)?;
    sink.set_property("sync", false)?;

    bin.add_many(&[&queue, &transform, &sink])?;
    common::add_bin_ghost_pad(&bin, &queue, "sink")?;
    queue.link(&transform)?;
    transform.link(&sink)?;

    Ok(bin)
}
