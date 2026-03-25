use std::cell::Cell;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/sidebar_row.ui")]
    pub(crate) struct SidebarRow {
        #[template_child]
        pub(super) channel_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) channel_index_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) unread_badge: TemplateChild<gtk::Label>,
        pub(super) channel_index: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "PaplSidebarRow";
        type Type = super::SidebarRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SidebarRow {}
    impl WidgetImpl for SidebarRow {}
    impl ListBoxRowImpl for SidebarRow {}
}

glib::wrapper! {
    pub(crate) struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible;
}

impl SidebarRow {
    pub(crate) fn new(channel: &model::Channel) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();

        imp.channel_name_label.set_label(&channel.name());
        imp.channel_index_label
            .set_label(&format!("Ch {}", channel.index()));
        imp.channel_index.set(channel.index());

        // Update unread badge
        obj.update_unread(channel.unread_count());

        // Listen for unread changes
        channel.connect_notify_local(
            Some("unread-count"),
            clone!(@weak obj => move |channel, _| {
                obj.update_unread(channel.unread_count());
            }),
        );

        obj
    }

    fn update_unread(&self, count: u32) {
        let badge = &self.imp().unread_badge;
        if count > 0 {
            badge.set_label(&format!("{}", count.min(99)));
            badge.set_visible(true);
        } else {
            badge.set_visible(false);
        }
    }

    pub(crate) fn channel_index(&self) -> u32 {
        self.imp().channel_index.get()
    }
}
