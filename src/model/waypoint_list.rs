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
