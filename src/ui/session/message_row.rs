use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/message_row.ui")]
    pub(crate) struct MessageRow {
        #[template_child]
        pub(super) sender_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) text_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) time_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) message_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) delivery_icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "PaplMessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageRow {}
    impl WidgetImpl for MessageRow {}
    impl BoxImpl for MessageRow {}
}

glib::wrapper! {
    pub(crate) struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible;
}

impl MessageRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn set_message(&self, message: &model::MeshMessage) {
        let imp = self.imp();

        let sender_name = message.sender_name();
        imp.sender_label.set_label(&sender_name);
        imp.text_label.set_label(&message.text());
        imp.time_label.set_label(&utils::format_timestamp(message.timestamp()));

        // Radio info
        let hops = message.hops();
        let snr = message.snr();
        let rssi = message.rssi();
        if snr != 0.0 || rssi != 0 {
            imp.info_label.set_label(&format!(
                "SNR: {:.1} | RSSI: {} | Hops: {}",
                snr, rssi, hops
            ));
            imp.info_label.set_visible(true);
        } else {
            imp.info_label.set_visible(false);
        }

        // Style outgoing messages differently
        match message.direction() {
            model::MessageDirection::Outgoing => {
                imp.message_box.set_halign(gtk::Align::End);
                imp.message_box.add_css_class("accent");
                imp.sender_label.set_label("You");
            }
            model::MessageDirection::Incoming => {
                imp.message_box.set_halign(gtk::Align::Start);
                imp.message_box.remove_css_class("accent");
            }
        }

        // Delivery status icon (outgoing only)
        self.update_delivery_icon(message.delivery_status());

        if message.direction() == model::MessageDirection::Outgoing {
            imp.delivery_icon.set_visible(true);
            // Listen for delivery status changes
            message.connect_notify_local(
                Some("delivery-status"),
                clone!(@weak self as obj => move |msg, _| {
                    obj.update_delivery_icon(msg.delivery_status());
                }),
            );
        } else {
            imp.delivery_icon.set_visible(false);
        }
    }

    fn update_delivery_icon(&self, status: model::DeliveryStatus) {
        let icon = &self.imp().delivery_icon;
        match status {
            model::DeliveryStatus::None => {
                icon.set_visible(false);
            }
            model::DeliveryStatus::Sending => {
                icon.set_icon_name(Some("emblem-synchronizing-symbolic"));
                icon.set_tooltip_text(Some("Sending..."));
                icon.remove_css_class("success");
                icon.remove_css_class("error");
                icon.set_visible(true);
            }
            model::DeliveryStatus::Delivered => {
                icon.set_icon_name(Some("emblem-ok-symbolic"));
                icon.set_tooltip_text(Some("Delivered"));
                icon.add_css_class("success");
                icon.remove_css_class("error");
                icon.set_visible(true);
            }
            model::DeliveryStatus::Failed => {
                icon.set_icon_name(Some("dialog-warning-symbolic"));
                icon.set_tooltip_text(Some("Delivery failed"));
                icon.remove_css_class("success");
                icon.add_css_class("error");
                icon.set_visible(true);
            }
        }
    }
}
