use adw::subclass::prelude::*;
use glib::clone;
use glib::subclass::Signal;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/sidebar.ui")]
    pub(crate) struct Sidebar {
        #[template_child]
        pub(super) channel_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) node_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) connection_info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) loading_spinner: TemplateChild<gtk::Spinner>,
        pub(super) device: RefCell<Option<model::Device>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "PaplSidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ui::SidebarRow::ensure_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("channel-selected")
                    .param_types([u32::static_type()])
                    .build()]
            })
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BinImpl for Sidebar {}
}

glib::wrapper! {
    pub(crate) struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Sidebar {
    pub(crate) fn set_device(&self, device: &model::Device) {
        let imp = self.imp();
        imp.device.replace(Some(device.clone()));

        // Set initial connection info
        imp.connection_info_label
            .set_label(&device.connection_info());

        self.update_channels(device);
        self.update_node_count(device);

        // Listen for status changes
        device.connect_notify_local(
            Some("status-message"),
            clone!(@weak self as obj => move |device, _| {
                obj.imp().status_label.set_label(&device.status_message());
            }),
        );

        // Listen for config-loading changes
        device.connect_notify_local(
            Some("config-loading"),
            clone!(@weak self as obj => move |device, _| {
                obj.imp().loading_spinner.set_spinning(device.config_loading());
            }),
        );

        // Show initial loading state
        imp.loading_spinner.set_spinning(device.config_loading());
        imp.status_label.set_label(&device.status_message());

        // Listen for channel changes
        device.connect_local(
            "channels-changed",
            false,
            clone!(@weak self as obj, @weak device => @default-return None, move |_| {
                obj.update_channels(&device);
                None
            }),
        );

        // Update node count when node list changes
        device.nodes().connect_items_changed(
            clone!(@weak self as obj, @weak device => move |_, _, _, _| {
                obj.update_node_count(&device);
            }),
        );

        // Handle channel selection
        imp.channel_list_box.connect_row_activated(
            clone!(@weak self as obj => move |_, row| {
                if let Some(sidebar_row) = row.downcast_ref::<ui::SidebarRow>() {
                    obj.emit_by_name::<()>("channel-selected", &[&sidebar_row.channel_index()]);
                }
            }),
        );
    }

    fn update_channels(&self, device: &model::Device) {
        let imp = self.imp();
        let list_box = &imp.channel_list_box;

        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        for channel in device.channels() {
            if channel.is_active() {
                let row = ui::SidebarRow::new(&channel);
                list_box.append(&row);
            }
        }
    }

    fn update_node_count(&self, device: &model::Device) {
        let count = device.nodes().len();
        self.imp()
            .node_count_label
            .set_label(&format!(
                "{} node{}",
                count,
                if count == 1 { "" } else { "s" }
            ));
    }
}
