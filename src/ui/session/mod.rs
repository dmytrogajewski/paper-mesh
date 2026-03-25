mod add_channel_dialog;
mod content;
mod map_view;
mod node_row;
mod sidebar;
mod sidebar_row;
mod message_row;

use adw::prelude::AdwDialogExt;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;

pub(crate) use self::add_channel_dialog::AddChannelDialog;
pub(crate) use self::content::Content;
pub(crate) use self::map_view::MapView;
pub(crate) use self::message_row::MessageRow;
pub(crate) use self::node_row::NodeRow;
pub(crate) use self::sidebar::Sidebar;
pub(crate) use self::sidebar_row::SidebarRow;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/mod.ui")]
    pub(crate) struct Session {
        #[template_child]
        pub(super) split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub(super) sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub(super) content: TemplateChild<Content>,
        #[template_child]
        pub(super) header_title: TemplateChild<adw::WindowTitle>,
        pub(super) device: RefCell<Option<model::Device>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "PaplSession";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Sidebar::ensure_type();
            Content::ensure_type();
            Self::bind_template(klass);

            klass.install_action("session.disconnect", None, move |obj, _, _| {
                if let Some(device) = obj.imp().device.borrow().as_ref() {
                    device.disconnect();
                }
            });

            klass.install_action("session.add-channel", None, move |obj, _, _| {
                obj.show_add_channel_dialog();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Session {}
    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub(crate) struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Session {
    pub(crate) fn set_device(&self, device: &model::Device) {
        let imp = self.imp();

        imp.sidebar.set_device(device);
        imp.device.replace(Some(device.clone()));

        // When a channel is selected in the sidebar, show it in the content area
        imp.sidebar.connect_local(
            "channel-selected",
            false,
            clone!(@weak self as obj => @default-return None, move |values| {
                let channel_index: u32 = values[1].get().unwrap();
                if let Some(device) = obj.imp().device.borrow().as_ref() {
                    if let Some(channel) = device.channel(channel_index) {
                        obj.imp().content.set_channel(device, &channel);
                        obj.imp().header_title.set_title(&channel.name());
                        obj.imp().split_view.set_show_content(true);
                    }
                }
                None
            }),
        );

        // When a node is selected, set up DM to that node on the primary channel
        imp.sidebar.connect_local(
            "node-selected",
            false,
            clone!(@weak self as obj => @default-return None, move |values| {
                let node_num: u32 = values[1].get().unwrap();
                if let Some(device) = obj.imp().device.borrow().as_ref() {
                    // Use primary channel (index 0) for DMs
                    if let Some(channel) = device.channel(0) {
                        obj.imp().content.set_channel(device, &channel);
                        obj.imp().content.set_dm_target(node_num, device);

                        let name = if let Some(node) = device.nodes().find_by_num(node_num) {
                            format!("DM: {}", node.display_name())
                        } else {
                            format!("DM: !{:08x}", node_num)
                        };
                        obj.imp().header_title.set_title(&name);
                        obj.imp().split_view.set_show_content(true);
                    }
                }
                None
            }),
        );
    }

    fn show_add_channel_dialog(&self) {
        let device = self.imp().device.borrow();
        let Some(device) = device.as_ref() else {
            return;
        };
        let dialog = AddChannelDialog::new(device);
        dialog.present(self);
    }
}
