use std::cell::Cell;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/content.ui")]
    pub(crate) struct Content {
        #[template_child]
        pub(super) message_list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub(super) message_entry: TemplateChild<ui::MessageEntry>,
        #[template_child]
        pub(super) empty_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) dm_bar: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) dm_target_label: TemplateChild<gtk::Label>,
        pub(super) device: RefCell<Option<model::Device>>,
        pub(super) channel: RefCell<Option<model::Channel>>,
        /// DM target node. 0xFFFFFFFF = broadcast
        pub(super) dm_target: Cell<u32>,
        /// Signal handler IDs to disconnect on channel switch
        pub(super) signal_handlers: RefCell<Vec<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Content {
        const NAME: &'static str = "PaplContent";
        type Type = super::Content;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ui::MessageRow::ensure_type();
            ui::MessageEntry::ensure_type();
            Self::bind_template(klass);

            klass.install_action("content.send-message", None, move |obj, _, _| {
                obj.send_message();
            });

            klass.install_action("content.clear-dm", None, move |obj, _, _| {
                obj.clear_dm_target();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Content {
        fn constructed(&self) {
            self.parent_constructed();
            self.dm_target.set(0xFFFFFFFF);

            // Set up the factory once — it's stateless and reusable
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                list_item.set_child(Some(&ui::MessageRow::new()));
            });
            factory.connect_bind(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(message) = list_item.item().and_downcast::<model::MeshMessage>() {
                    if let Some(row) = list_item.child().and_downcast::<ui::MessageRow>() {
                        row.set_message(&message);
                    }
                }
            });
            self.message_list_view.set_factory(Some(&factory));
        }
    }

    impl WidgetImpl for Content {}
    impl BinImpl for Content {}
}

glib::wrapper! {
    pub(crate) struct Content(ObjectSubclass<imp::Content>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Content {
    pub(crate) fn set_channel(&self, device: &model::Device, channel: &model::Channel) {
        let imp = self.imp();

        // Disconnect old signal handlers
        if let Some(old_channel) = imp.channel.borrow().as_ref() {
            let old_messages = old_channel.messages();
            for handler_id in imp.signal_handlers.borrow_mut().drain(..) {
                old_messages.disconnect(handler_id);
            }
        }

        imp.device.replace(Some(device.clone()));
        imp.channel.replace(Some(channel.clone()));

        let messages = channel.messages();

        // Set new model
        let selection = gtk::NoSelection::new(Some(messages.clone()));
        imp.message_list_view.set_model(Some(&selection));

        // Auto-scroll to bottom when new messages arrive
        let list_view = imp.message_list_view.clone();
        let h1 = messages.connect_items_changed(
            clone!(@weak list_view => move |model, _, _, _| {
                let n = model.n_items();
                if n > 0 {
                    list_view.scroll_to(n - 1, gtk::ListScrollFlags::NONE, None);
                }
            }),
        );

        // Show message list or empty state
        let has_messages = messages.n_items() > 0;
        imp.content_stack
            .set_visible_child_name(if has_messages { "messages" } else { "empty" });

        let content_stack = imp.content_stack.clone();
        let h2 = messages.connect_items_changed(
            clone!(@weak content_stack as stack => move |model, _, _, _| {
                if model.n_items() > 0 {
                    stack.set_visible_child_name("messages");
                }
            }),
        );

        // Store handler IDs for cleanup
        imp.signal_handlers.borrow_mut().extend([h1, h2]);

        // Scroll to bottom if there are existing messages
        if has_messages {
            let n = messages.n_items();
            imp.message_list_view
                .scroll_to(n - 1, gtk::ListScrollFlags::NONE, None);
        }

        // Reset DM target when switching channels
        self.clear_dm_target();
    }

    /// Set a DM target node
    pub(crate) fn set_dm_target(&self, node_num: u32, device: &model::Device) {
        let imp = self.imp();
        imp.dm_target.set(node_num);
        imp.dm_bar.set_visible(true);

        let name = if let Some(node) = device.nodes().find_by_num(node_num) {
            node.display_name()
        } else {
            format!("!{:08x}", node_num)
        };
        imp.dm_target_label.set_label(&name);
    }

    fn clear_dm_target(&self) {
        let imp = self.imp();
        imp.dm_target.set(0xFFFFFFFF);
        imp.dm_bar.set_visible(false);
    }

    fn send_message(&self) {
        let imp = self.imp();
        let text = imp.message_entry.text();
        if text.is_empty() {
            return;
        }

        let device = imp.device.borrow();
        let channel = imp.channel.borrow();
        let destination = imp.dm_target.get();

        if let (Some(device), Some(channel)) = (device.as_ref(), channel.as_ref()) {
            device.send_text(&text, channel.index(), destination);
            imp.message_entry.clear();
        }
    }
}
