use std::cell::Cell;
use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::types::NodeId;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Node {
        pub(super) num: Cell<NodeId>,
        pub(super) long_name: RefCell<String>,
        pub(super) short_name: RefCell<String>,
        pub(super) hw_model: RefCell<String>,
        pub(super) battery_level: Cell<u32>,
        pub(super) snr: Cell<f32>,
        pub(super) last_heard: Cell<u32>,
        pub(super) latitude: Cell<f64>,
        pub(super) longitude: Cell<f64>,
        pub(super) altitude: Cell<i32>,
        pub(super) is_online: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Node {
        const NAME: &'static str = "MeshNode";
        type Type = super::Node;
    }

    impl ObjectImpl for Node {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("num").read_only().build(),
                    glib::ParamSpecString::builder("long-name").read_only().build(),
                    glib::ParamSpecString::builder("short-name").read_only().build(),
                    glib::ParamSpecString::builder("hw-model").read_only().build(),
                    glib::ParamSpecUInt::builder("battery-level").read_only().build(),
                    glib::ParamSpecFloat::builder("snr").read_only().build(),
                    glib::ParamSpecUInt::builder("last-heard").read_only().build(),
                    glib::ParamSpecBoolean::builder("is-online").read_only().build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "num" => obj.num().to_value(),
                "long-name" => obj.long_name().to_value(),
                "short-name" => obj.short_name().to_value(),
                "hw-model" => obj.hw_model().to_value(),
                "battery-level" => obj.battery_level().to_value(),
                "snr" => obj.snr().to_value(),
                "last-heard" => obj.last_heard().to_value(),
                "is-online" => obj.is_online().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Node(ObjectSubclass<imp::Node>);
}

impl Node {
    pub(crate) fn new(num: NodeId) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().num.set(num);
        obj
    }

    pub(crate) fn num(&self) -> NodeId {
        self.imp().num.get()
    }

    pub(crate) fn long_name(&self) -> String {
        self.imp().long_name.borrow().clone()
    }

    pub(crate) fn short_name(&self) -> String {
        self.imp().short_name.borrow().clone()
    }

    pub(crate) fn hw_model(&self) -> String {
        self.imp().hw_model.borrow().clone()
    }

    pub(crate) fn battery_level(&self) -> u32 {
        self.imp().battery_level.get()
    }

    pub(crate) fn snr(&self) -> f32 {
        self.imp().snr.get()
    }

    pub(crate) fn last_heard(&self) -> u32 {
        self.imp().last_heard.get()
    }

    pub(crate) fn is_online(&self) -> bool {
        self.imp().is_online.get()
    }

    pub(crate) fn set_long_name(&self, name: &str) {
        self.imp().long_name.replace(name.to_string());
        self.notify("long-name");
    }

    pub(crate) fn set_short_name(&self, name: &str) {
        self.imp().short_name.replace(name.to_string());
        self.notify("short-name");
    }

    pub(crate) fn set_hw_model(&self, model: &str) {
        self.imp().hw_model.replace(model.to_string());
        self.notify("hw-model");
    }

    pub(crate) fn set_battery_level(&self, level: u32) {
        self.imp().battery_level.set(level);
        self.notify("battery-level");
    }

    pub(crate) fn set_snr(&self, snr: f32) {
        self.imp().snr.set(snr);
        self.notify("snr");
    }

    pub(crate) fn set_last_heard(&self, ts: u32) {
        self.imp().last_heard.set(ts);
        self.notify("last-heard");
    }

    pub(crate) fn latitude(&self) -> f64 {
        self.imp().latitude.get()
    }

    pub(crate) fn longitude(&self) -> f64 {
        self.imp().longitude.get()
    }

    pub(crate) fn altitude(&self) -> i32 {
        self.imp().altitude.get()
    }

    pub(crate) fn set_position(&self, lat: f64, lon: f64, alt: i32) {
        self.imp().latitude.set(lat);
        self.imp().longitude.set(lon);
        self.imp().altitude.set(alt);
    }

    pub(crate) fn set_is_online(&self, online: bool) {
        self.imp().is_online.set(online);
        self.notify("is-online");
    }

    /// Display name: long_name if available, otherwise hex node number
    pub(crate) fn display_name(&self) -> String {
        let name = self.long_name();
        if name.is_empty() {
            format!("!{:08x}", self.num())
        } else {
            name
        }
    }
}
