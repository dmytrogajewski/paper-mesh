use std::cell::RefCell;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::MeshMessage;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct MessageList {
        pub(super) messages: RefCell<Vec<MeshMessage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageList {
        const NAME: &'static str = "MeshMessageList";
        type Type = super::MessageList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for MessageList {}

    impl ListModelImpl for MessageList {
        fn item_type(&self) -> glib::Type {
            MeshMessage::static_type()
        }

        fn n_items(&self) -> u32 {
            self.messages.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.messages
                .borrow()
                .get(position as usize)
                .map(|m| m.clone().upcast())
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageList(ObjectSubclass<imp::MessageList>)
        @implements gio::ListModel;
}

impl Default for MessageList {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MessageList {
    pub(crate) fn append(&self, message: MeshMessage) {
        let pos = {
            let mut messages = self.imp().messages.borrow_mut();
            let pos = messages.len() as u32;
            messages.push(message);
            pos
        };
        self.items_changed(pos, 0, 1);
    }

    pub(crate) fn len(&self) -> usize {
        self.imp().messages.borrow().len()
    }
}
