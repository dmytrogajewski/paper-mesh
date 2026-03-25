mod channel;
mod device;
mod message;
mod message_list;
pub(crate) mod message_store;
mod node;
mod node_list;
pub(crate) mod range_test;
pub(crate) mod telemetry;
mod waypoint;
mod waypoint_list;

use gtk::glib;

pub(crate) use self::channel::Channel;
pub(crate) use self::device::Device;
pub(crate) use self::device::DeviceState;
mod canned_messages;

pub(crate) use self::canned_messages::CannedMessages;
pub(crate) use self::message::DeliveryStatus;
pub(crate) use self::message::MeshMessage;
pub(crate) use self::message::MessageDirection;
pub(crate) use self::message_list::MessageList;
pub(crate) use self::node::Node;
pub(crate) use self::node_list::NodeList;
pub(crate) use self::waypoint::Waypoint;
pub(crate) use self::waypoint_list::WaypointList;

/// Connection method for the Meshtastic device
#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedConnectionMethod")]
pub(crate) enum ConnectionMethod {
    Serial(String),
    Tcp(String),
}
