use std::cell::Cell;
use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::MessageList;
use crate::types::ChannelIndex;

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Channel {
        pub(super) index: Cell<ChannelIndex>,
        pub(super) name: RefCell<String>,
        pub(super) role: Cell<u32>,
        pub(super) messages: OnceCell<MessageList>,
        pub(super) unread_count: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Channel {
        const NAME: &'static str = "MeshChannel";
        type Type = super::Channel;
    }

    impl ObjectImpl for Channel {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("index").read_only().build(),
                    glib::ParamSpecString::builder("name").read_only().build(),
                    glib::ParamSpecUInt::builder("role").read_only().build(),
                    glib::ParamSpecUInt::builder("unread-count").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "index" => obj.index().to_value(),
                "name" => obj.name().to_value(),
                "role" => obj.role().to_value(),
                "unread-count" => obj.unread_count().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Channel(ObjectSubclass<imp::Channel>);
}

impl Channel {
    pub(crate) fn new(index: ChannelIndex) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().index.set(index);
        obj.imp().messages.set(MessageList::default()).unwrap();
        obj
    }

    pub(crate) fn index(&self) -> ChannelIndex {
        self.imp().index.get()
    }

    pub(crate) fn name(&self) -> String {
        let name = self.imp().name.borrow().clone();
        if name.is_empty() {
            if self.index() == 0 {
                "Primary".to_string()
            } else {
                format!("Channel {}", self.index())
            }
        } else {
            name
        }
    }

    pub(crate) fn role(&self) -> u32 {
        self.imp().role.get()
    }

    pub(crate) fn messages(&self) -> &MessageList {
        self.imp().messages.get().unwrap()
    }

    pub(crate) fn set_name(&self, name: &str) {
        self.imp().name.replace(name.to_string());
        self.notify("name");
    }

    pub(crate) fn set_role(&self, role: u32) {
        self.imp().role.set(role);
        self.notify("role");
    }

    pub(crate) fn unread_count(&self) -> u32 {
        self.imp().unread_count.get()
    }

    pub(crate) fn increment_unread(&self) {
        let count = self.imp().unread_count.get() + 1;
        self.imp().unread_count.set(count);
        self.notify("unread-count");
    }

    pub(crate) fn clear_unread(&self) {
        self.imp().unread_count.set(0);
        self.notify("unread-count");
    }

    /// Whether this channel is active (has a role other than DISABLED=0)
    pub(crate) fn is_active(&self) -> bool {
        self.role() != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_gtk() { crate::test_helpers::init_gtk(); }

    #[test]
    fn test_channel_creation() {
        init_gtk();
        let ch = Channel::new(0);
        assert_eq!(ch.index(), 0);
        assert_eq!(ch.role(), 0);
        assert!(!ch.is_active());
    }

    #[test]
    fn test_primary_channel_default_name() {
        init_gtk();
        let ch = Channel::new(0);
        assert_eq!(ch.name(), "Primary");
    }

    #[test]
    fn test_secondary_channel_default_name() {
        init_gtk();
        let ch = Channel::new(3);
        assert_eq!(ch.name(), "Channel 3");
    }

    #[test]
    fn test_custom_name() {
        init_gtk();
        let ch = Channel::new(1);
        ch.set_name("MyChannel");
        assert_eq!(ch.name(), "MyChannel");
    }

    #[test]
    fn test_active_role() {
        init_gtk();
        let ch = Channel::new(0);
        assert!(!ch.is_active());

        ch.set_role(1); // PRIMARY
        assert!(ch.is_active());

        ch.set_role(2); // SECONDARY
        assert!(ch.is_active());

        ch.set_role(0); // DISABLED
        assert!(!ch.is_active());
    }

    #[test]
    fn test_messages_list() {
        init_gtk();
        let ch = Channel::new(0);
        assert_eq!(ch.messages().len(), 0);
    }
}
