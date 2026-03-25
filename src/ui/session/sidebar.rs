use adw::subclass::prelude::*;
use glib::clone;
use glib::subclass::Signal;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

use super::map_view::MapView;
use super::node_row::NodeRow;

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
        pub(super) node_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) node_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) connection_info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) loading_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub(super) map_view: TemplateChild<MapView>,
        pub(super) device: RefCell<Option<model::Device>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "PaplSidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ui::SidebarRow::ensure_type();
            NodeRow::ensure_type();
            MapView::ensure_type();
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
                vec![
                    Signal::builder("channel-selected")
                        .param_types([u32::static_type()])
                        .build(),
                    Signal::builder("node-selected")
                        .param_types([u32::static_type()])
                        .build(),
                ]
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

        imp.connection_info_label
            .set_label(&device.connection_info());

        self.update_channels(device);
        self.update_nodes(device);
        self.update_node_count(device);

        // Set up map
        imp.map_view.set_device(device);

        // Listen for status changes
        device.connect_notify_local(
            Some("status-message"),
            clone!(@weak self as obj => move |device, _| {
                obj.imp().status_label.set_label(&device.status_message());
            }),
        );

        device.connect_notify_local(
            Some("config-loading"),
            clone!(@weak self as obj => move |device, _| {
                obj.imp().loading_spinner.set_spinning(device.config_loading());
            }),
        );

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

        // Update node list and count when nodes change
        device.nodes().connect_items_changed(
            clone!(@weak self as obj, @weak device => move |_, _, _, _| {
                obj.update_node_count(&device);
                obj.update_nodes(&device);
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

        // Handle node selection (for DM)
        imp.node_list_box.connect_row_activated(
            clone!(@weak self as obj => move |_, row| {
                // Get the NodeRow from the ListBoxRow child
                if let Some(child) = row.child() {
                    if let Some(node_row) = child.downcast_ref::<NodeRow>() {
                        obj.emit_by_name::<()>("node-selected", &[&node_row.node_num()]);
                    }
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

    fn update_nodes(&self, device: &model::Device) {
        let imp = self.imp();
        let list_box = &imp.node_list_box;

        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let nodes = device.nodes();
        for i in 0..nodes.n_items() {
            let Some(obj) = nodes.item(i) else { continue };
            let node = obj.downcast_ref::<model::Node>().unwrap();

            let node_row = NodeRow::new();
            node_row.set_node(node);

            let list_row = gtk::ListBoxRow::new();
            list_row.set_child(Some(&node_row));
            list_box.append(&list_row);
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
