use std::cell::Cell;
use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::types::NodeId;
use crate::types::PacketId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "MessageDirection")]
pub(crate) enum MessageDirection {
    #[default]
    Incoming,
    Outgoing,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct MeshMessage {
        pub(super) packet_id: Cell<PacketId>,
        pub(super) from_node: Cell<NodeId>,
        pub(super) to_node: Cell<NodeId>,
        pub(super) channel_index: Cell<u32>,
        pub(super) text: RefCell<String>,
        pub(super) timestamp: Cell<u32>,
        pub(super) direction: Cell<MessageDirection>,
        pub(super) snr: Cell<f32>,
        pub(super) rssi: Cell<i32>,
        pub(super) hop_start: Cell<u32>,
        pub(super) hop_limit: Cell<u32>,
        pub(super) sender_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MeshMessage {
        const NAME: &'static str = "MeshMessage";
        type Type = super::MeshMessage;
    }

    impl ObjectImpl for MeshMessage {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("packet-id").read_only().build(),
                    glib::ParamSpecUInt::builder("from-node").read_only().build(),
                    glib::ParamSpecString::builder("text").read_only().build(),
                    glib::ParamSpecUInt::builder("timestamp").read_only().build(),
                    glib::ParamSpecString::builder("sender-name").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "packet-id" => obj.packet_id().to_value(),
                "from-node" => obj.from_node().to_value(),
                "text" => obj.text().to_value(),
                "timestamp" => obj.timestamp().to_value(),
                "sender-name" => obj.sender_name().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MeshMessage(ObjectSubclass<imp::MeshMessage>);
}

impl MeshMessage {
    pub(crate) fn new(
        packet_id: PacketId,
        from_node: NodeId,
        to_node: NodeId,
        channel_index: u32,
        text: &str,
        timestamp: u32,
        direction: MessageDirection,
    ) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        imp.packet_id.set(packet_id);
        imp.from_node.set(from_node);
        imp.to_node.set(to_node);
        imp.channel_index.set(channel_index);
        imp.text.replace(text.to_string());
        imp.timestamp.set(timestamp);
        imp.direction.set(direction);
        obj
    }

    pub(crate) fn packet_id(&self) -> PacketId {
        self.imp().packet_id.get()
    }

    pub(crate) fn from_node(&self) -> NodeId {
        self.imp().from_node.get()
    }

    pub(crate) fn to_node(&self) -> NodeId {
        self.imp().to_node.get()
    }

    pub(crate) fn channel_index(&self) -> u32 {
        self.imp().channel_index.get()
    }

    pub(crate) fn text(&self) -> String {
        self.imp().text.borrow().clone()
    }

    pub(crate) fn timestamp(&self) -> u32 {
        self.imp().timestamp.get()
    }

    pub(crate) fn direction(&self) -> MessageDirection {
        self.imp().direction.get()
    }

    pub(crate) fn snr(&self) -> f32 {
        self.imp().snr.get()
    }

    pub(crate) fn rssi(&self) -> i32 {
        self.imp().rssi.get()
    }

    pub(crate) fn sender_name(&self) -> String {
        self.imp().sender_name.borrow().clone()
    }

    pub(crate) fn set_sender_name(&self, name: &str) {
        self.imp().sender_name.replace(name.to_string());
        self.notify("sender-name");
    }

    pub(crate) fn set_radio_info(&self, snr: f32, rssi: i32, hop_start: u32, hop_limit: u32) {
        let imp = self.imp();
        imp.snr.set(snr);
        imp.rssi.set(rssi);
        imp.hop_start.set(hop_start);
        imp.hop_limit.set(hop_limit);
    }

    pub(crate) fn hops(&self) -> u32 {
        self.imp().hop_start.get().saturating_sub(self.imp().hop_limit.get())
    }
}
