use std::cell::RefCell;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::Waypoint;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct WaypointList {
        pub(super) waypoints: RefCell<Vec<Waypoint>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WaypointList {
        const NAME: &'static str = "MeshWaypointList";
        type Type = super::WaypointList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for WaypointList {}

    impl ListModelImpl for WaypointList {
        fn item_type(&self) -> glib::Type {
            Waypoint::static_type()
        }

        fn n_items(&self) -> u32 {
            self.waypoints.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.waypoints
                .borrow()
                .get(position as usize)
                .map(|w| w.clone().upcast())
        }
    }
}

glib::wrapper! {
    pub(crate) struct WaypointList(ObjectSubclass<imp::WaypointList>)
        @implements gio::ListModel;
}

impl Default for WaypointList {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl WaypointList {
    pub(crate) fn add_or_update(&self, waypoint: Waypoint) {
        let mut waypoints = self.imp().waypoints.borrow_mut();
        if let Some(pos) = waypoints.iter().position(|w| w.id() == waypoint.id()) {
            waypoints[pos] = waypoint;
            drop(waypoints);
            self.items_changed(pos as u32, 1, 1);
        } else {
            let pos = waypoints.len() as u32;
            waypoints.push(waypoint);
            drop(waypoints);
            self.items_changed(pos, 0, 1);
        }
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
        let list = WaypointList::default();
        assert_eq!(list.n_items(), 0);
    }

    #[test]
    fn test_add_waypoint() {
        init_gtk();
        let list = WaypointList::default();
        let wp = Waypoint::new(1, "WP1", "desc", 51.5, -0.1, 0, 0, 100);
        list.add_or_update(wp);
        assert_eq!(list.n_items(), 1);

        let item = list.item(0).unwrap();
        let wp = item.downcast_ref::<Waypoint>().unwrap();
        assert_eq!(wp.name(), "WP1");
    }

    #[test]
    fn test_update_existing_waypoint() {
        init_gtk();
        let list = WaypointList::default();

        let wp1 = Waypoint::new(1, "Old", "old desc", 0.0, 0.0, 0, 0, 0);
        list.add_or_update(wp1);

        let wp2 = Waypoint::new(1, "New", "new desc", 1.0, 1.0, 0, 0, 0);
        list.add_or_update(wp2);

        assert_eq!(list.n_items(), 1);
        let item = list.item(0).unwrap();
        let wp = item.downcast_ref::<Waypoint>().unwrap();
        assert_eq!(wp.name(), "New");
    }

    #[test]
    fn test_multiple_waypoints() {
        init_gtk();
        let list = WaypointList::default();
        list.add_or_update(Waypoint::new(1, "A", "", 0.0, 0.0, 0, 0, 0));
        list.add_or_update(Waypoint::new(2, "B", "", 0.0, 0.0, 0, 0, 0));
        list.add_or_update(Waypoint::new(3, "C", "", 0.0, 0.0, 0, 0, 0));
        assert_eq!(list.n_items(), 3);
    }
}
