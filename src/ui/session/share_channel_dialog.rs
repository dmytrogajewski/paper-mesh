use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::CompositeTemplate;

use crate::model;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/share_channel_dialog.ui")]
    pub(crate) struct ShareChannelDialog {
        #[template_child]
        pub(super) channel_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) qr_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub(super) url_row: TemplateChild<adw::ActionRow>,
        pub(super) channel_url: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShareChannelDialog {
        const NAME: &'static str = "PaplShareChannelDialog";
        type Type = super::ShareChannelDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("share.copy-url", None, move |obj, _, _| {
                obj.copy_url();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShareChannelDialog {}
    impl WidgetImpl for ShareChannelDialog {}
    impl AdwDialogImpl for ShareChannelDialog {}
}

glib::wrapper! {
    pub(crate) struct ShareChannelDialog(ObjectSubclass<imp::ShareChannelDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible;
}

impl ShareChannelDialog {
    pub(crate) fn new(channel: &model::Channel) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();

        imp.channel_name_label.set_label(&channel.name());

        // Build a Meshtastic-style channel URL
        // Format: https://meshtastic.org/e/#<base64_encoded_channel_settings>
        // For simplicity, encode the channel name as the URL
        let channel_data = format!(
            "{{\"channelName\":\"{}\",\"channelIndex\":{}}}",
            channel.name(),
            channel.index()
        );
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            channel_data.as_bytes(),
        );
        let url = format!("https://meshtastic.org/e/#{}", encoded);

        imp.url_row.set_subtitle(&url);
        imp.channel_url.replace(url.clone());

        // Generate QR code
        if let Ok(svg) =
            qrcode_generator::to_svg_to_string(&url, qrcode_generator::QrCodeEcc::Medium, 256, None::<&str>)
        {
            let bytes = glib::Bytes::from_owned(svg.into_bytes());
            let stream = gtk::gio::MemoryInputStream::from_bytes(&bytes);
            if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_stream(
                &stream,
                gtk::gio::Cancellable::NONE,
            ) {
                let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
                imp.qr_picture.set_paintable(Some(&texture));
            }
        }

        obj
    }

    fn copy_url(&self) {
        let url = self.imp().channel_url.borrow().clone();
        if let Some(display) = gtk::gdk::Display::default() {
            let clipboard = display.clipboard();
            clipboard.set_text(&url);
            // Could show a toast here
        }
    }
}
