mod channel;
mod device;
mod message;
mod message_list;
pub(crate) mod message_store;
mod node;
mod node_list;

use gtk::glib;

pub(crate) use self::channel::Channel;
pub(crate) use self::device::Device;
pub(crate) use self::device::DeviceState;
pub(crate) use self::message::MeshMessage;
pub(crate) use self::message::MessageDirection;
pub(crate) use self::message_list::MessageList;
pub(crate) use self::node::Node;
pub(crate) use self::node_list::NodeList;

/// Connection method for the Meshtastic device
#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedConnectionMethod")]
pub(crate) enum ConnectionMethod {
    Serial(String),
    Tcp(String),
}
