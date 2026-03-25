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

#[cfg(test)]
mod tests {
    use super::*;
    use gtk::prelude::*;

    fn init_gtk() { crate::test_helpers::init_gtk(); }

    #[test]
    fn test_empty_list() {
        init_gtk();
        let list = NodeList::default();
        assert_eq!(list.len(), 0);
        assert!(list.find_by_num(1).is_none());
        assert_eq!(list.n_items(), 0);
    }

    #[test]
    fn test_add_node() {
        init_gtk();
        let list = NodeList::default();
        let node = list.add_or_update(42);
        assert_eq!(node.num(), 42);
        assert_eq!(list.len(), 1);
        assert_eq!(list.n_items(), 1);
    }

    #[test]
    fn test_add_duplicate_returns_existing() {
        init_gtk();
        let list = NodeList::default();
        let n1 = list.add_or_update(10);
        n1.set_long_name("First");
        let n2 = list.add_or_update(10);
        assert_eq!(n2.long_name(), "First");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_find_by_num() {
        init_gtk();
        let list = NodeList::default();
        list.add_or_update(1);
        list.add_or_update(2);
        list.add_or_update(3);

        assert!(list.find_by_num(2).is_some());
        assert!(list.find_by_num(99).is_none());
    }

    #[test]
    fn test_list_model_item() {
        init_gtk();
        let list = NodeList::default();
        list.add_or_update(100);
        list.add_or_update(200);

        let item = list.item(0).unwrap();
        let node = item.downcast_ref::<Node>().unwrap();
        assert_eq!(node.num(), 100);

        let item = list.item(1).unwrap();
        let node = item.downcast_ref::<Node>().unwrap();
        assert_eq!(node.num(), 200);

        assert!(list.item(2).is_none());
    }
}
