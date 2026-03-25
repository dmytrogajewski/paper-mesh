use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::utils;

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/node_row.ui")]
    pub(crate) struct NodeRow {
        #[template_child]
        pub(super) short_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) long_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) hw_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) battery_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) last_heard_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) online_icon: TemplateChild<gtk::Image>,
        pub(super) node_num: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NodeRow {
        const NAME: &'static str = "PaplNodeRow";
        type Type = super::NodeRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NodeRow {}
    impl WidgetImpl for NodeRow {}
    impl BoxImpl for NodeRow {}
}

glib::wrapper! {
    pub(crate) struct NodeRow(ObjectSubclass<imp::NodeRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible;
}

impl NodeRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn set_node(&self, node: &model::Node) {
        let imp = self.imp();
        imp.node_num.set(node.num());

        let sn = node.short_name();
        imp.short_name_label.set_label(if sn.is_empty() {
            "?"
        } else {
            &sn
        });

        imp.long_name_label.set_label(&node.display_name());
        imp.hw_label.set_label(&node.hw_model());

        let bat = node.battery_level();
        if bat > 0 && bat <= 100 {
            imp.battery_label.set_label(&format!("{}%", bat));
            imp.battery_label.set_visible(true);
        } else {
            imp.battery_label.set_visible(false);
        }

        let lh = node.last_heard();
        if lh > 0 {
            imp.last_heard_label.set_label(&utils::format_timestamp(lh));
        } else {
            imp.last_heard_label.set_label("");
        }

        if node.is_online() {
            imp.online_icon.add_css_class("success");
            imp.online_icon.remove_css_class("dim-label");
        } else {
            imp.online_icon.remove_css_class("success");
            imp.online_icon.add_css_class("dim-label");
        }
    }

    pub(crate) fn node_num(&self) -> u32 {
        self.imp().node_num.get()
    }
}
