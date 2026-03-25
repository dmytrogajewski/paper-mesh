use std::cell::Cell;
use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Waypoint {
        pub(super) id: Cell<u32>,
        pub(super) name: RefCell<String>,
        pub(super) description: RefCell<String>,
        pub(super) latitude: Cell<f64>,
        pub(super) longitude: Cell<f64>,
        pub(super) expire: Cell<u32>,
        pub(super) locked_to: Cell<u32>,
        pub(super) from_node: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Waypoint {
        const NAME: &'static str = "MeshWaypoint";
        type Type = super::Waypoint;
    }

    impl ObjectImpl for Waypoint {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("id").read_only().build(),
                    glib::ParamSpecString::builder("name").read_only().build(),
                    glib::ParamSpecString::builder("description").read_only().build(),
                    glib::ParamSpecDouble::builder("latitude").read_only().build(),
                    glib::ParamSpecDouble::builder("longitude").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "id" => obj.id().to_value(),
                "name" => obj.name().to_value(),
                "description" => obj.description().to_value(),
                "latitude" => obj.latitude().to_value(),
                "longitude" => obj.longitude().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Waypoint(ObjectSubclass<imp::Waypoint>);
}

impl Waypoint {
    pub(crate) fn new(
        id: u32,
        name: &str,
        description: &str,
        lat: f64,
        lon: f64,
        expire: u32,
        locked_to: u32,
        from_node: u32,
    ) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        imp.id.set(id);
        imp.name.replace(name.to_string());
        imp.description.replace(description.to_string());
        imp.latitude.set(lat);
        imp.longitude.set(lon);
        imp.expire.set(expire);
        imp.locked_to.set(locked_to);
        imp.from_node.set(from_node);
        obj
    }

    pub(crate) fn id(&self) -> u32 { self.imp().id.get() }
    pub(crate) fn name(&self) -> String { self.imp().name.borrow().clone() }
    pub(crate) fn description(&self) -> String { self.imp().description.borrow().clone() }
    pub(crate) fn latitude(&self) -> f64 { self.imp().latitude.get() }
    pub(crate) fn longitude(&self) -> f64 { self.imp().longitude.get() }
    pub(crate) fn from_node(&self) -> u32 { self.imp().from_node.get() }
}
