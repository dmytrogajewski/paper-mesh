use std::cell::RefCell;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::Node;
use crate::types::NodeId;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct NodeList {
        pub(super) nodes: RefCell<Vec<Node>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NodeList {
        const NAME: &'static str = "MeshNodeList";
        type Type = super::NodeList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for NodeList {}

    impl ListModelImpl for NodeList {
        fn item_type(&self) -> glib::Type {
            Node::static_type()
        }

        fn n_items(&self) -> u32 {
            self.nodes.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.nodes
                .borrow()
                .get(position as usize)
                .map(|n| n.clone().upcast())
        }
    }
}

glib::wrapper! {
    pub(crate) struct NodeList(ObjectSubclass<imp::NodeList>)
        @implements gio::ListModel;
}

impl Default for NodeList {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl NodeList {
    pub(crate) fn find_by_num(&self, num: NodeId) -> Option<Node> {
        self.imp()
            .nodes
            .borrow()
            .iter()
            .find(|n| n.num() == num)
            .cloned()
    }

    pub(crate) fn add_or_update(&self, num: NodeId) -> Node {
        if let Some(node) = self.find_by_num(num) {
            return node;
        }
        let node = Node::new(num);
        let pos = {
            let mut nodes = self.imp().nodes.borrow_mut();
            let pos = nodes.len() as u32;
            nodes.push(node.clone());
            pos
        };
        self.items_changed(pos, 0, 1);
        node
    }

    pub(crate) fn len(&self) -> usize {
        self.imp().nodes.borrow().len()
    }
}
