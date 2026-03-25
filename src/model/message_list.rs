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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MessageDirection;
    use gtk::prelude::*;

    fn init_gtk() { crate::test_helpers::init_gtk(); }

    #[test]
    fn test_empty_list() {
        init_gtk();
        let list = MessageList::default();
        assert_eq!(list.len(), 0);
        assert_eq!(list.n_items(), 0);
    }

    #[test]
    fn test_append() {
        init_gtk();
        let list = MessageList::default();
        let msg = MeshMessage::new(1, 0, 0, 0, "hello", 0, MessageDirection::Incoming);
        list.append(msg);
        assert_eq!(list.len(), 1);
        assert_eq!(list.n_items(), 1);
    }

    #[test]
    fn test_list_model_item() {
        init_gtk();
        let list = MessageList::default();
        list.append(MeshMessage::new(10, 0, 0, 0, "first", 0, MessageDirection::Incoming));
        list.append(MeshMessage::new(20, 0, 0, 0, "second", 0, MessageDirection::Outgoing));

        let item = list.item(0).unwrap();
        let msg = item.downcast_ref::<MeshMessage>().unwrap();
        assert_eq!(msg.text(), "first");

        let item = list.item(1).unwrap();
        let msg = item.downcast_ref::<MeshMessage>().unwrap();
        assert_eq!(msg.text(), "second");

        assert!(list.item(2).is_none());
    }

    #[test]
    fn test_multiple_appends() {
        init_gtk();
        let list = MessageList::default();
        for i in 0..50 {
            list.append(MeshMessage::new(i, 0, 0, 0, &format!("msg {i}"), 0, MessageDirection::Incoming));
        }
        assert_eq!(list.len(), 50);
    }
}
