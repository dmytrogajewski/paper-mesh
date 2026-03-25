use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/components/message_entry.ui")]
    pub(crate) struct MessageEntry {
        #[template_child]
        pub(super) text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub(super) char_count: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageEntry {
        const NAME: &'static str = "PaplMessageEntry";
        type Type = super::MessageEntry;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageEntry {
        fn constructed(&self) {
            self.parent_constructed();

            let char_count = self.char_count.clone();
            self.text_view.buffer().connect_changed(move |buffer| {
                let len = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).len();
                char_count.set_label(&format!("{}/228", len));
                if len > 228 {
                    char_count.add_css_class("error");
                } else {
                    char_count.remove_css_class("error");
                }
            });

            // Send on Enter (Shift+Enter for newline)
            let text_view = self.text_view.clone();
            let key_controller = gtk::EventControllerKey::new();
            key_controller.connect_key_pressed(
                glib::clone!(@weak text_view => @default-return glib::Propagation::Proceed, move |_, key, _, modifier| {
                    if key == gtk::gdk::Key::Return && !modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
                        text_view.activate_action("content.send-message", None).ok();
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                }),
            );
            self.text_view.add_controller(key_controller);
        }
    }

    impl WidgetImpl for MessageEntry {}
    impl BoxImpl for MessageEntry {}
}

glib::wrapper! {
    pub(crate) struct MessageEntry(ObjectSubclass<imp::MessageEntry>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible;
}

impl MessageEntry {
    pub(crate) fn text(&self) -> String {
        let buffer = self.imp().text_view.buffer();
        buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string()
    }

    pub(crate) fn clear(&self) {
        self.imp().text_view.buffer().set_text("");
    }
}
