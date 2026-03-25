use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/session/add_channel_dialog.ui")]
    pub(crate) struct AddChannelDialog {
        #[template_child]
        pub(super) name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) slot_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) slot_model: TemplateChild<gtk::StringList>,
        #[template_child]
        pub(super) psk_type_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) create_button: TemplateChild<gtk::Button>,
        pub(super) device: RefCell<Option<model::Device>>,
        pub(super) free_slots: RefCell<Vec<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddChannelDialog {
        const NAME: &'static str = "PaplAddChannelDialog";
        type Type = super::AddChannelDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddChannelDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.cancel_button.connect_clicked(
                clone!(@weak obj => move |_| {
                    obj.close();
                }),
            );

            self.create_button.connect_clicked(
                clone!(@weak obj => move |_| {
                    obj.on_create();
                }),
            );
        }
    }

    impl WidgetImpl for AddChannelDialog {}
    impl AdwDialogImpl for AddChannelDialog {}
}

glib::wrapper! {
    pub(crate) struct AddChannelDialog(ObjectSubclass<imp::AddChannelDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible;
}

impl AddChannelDialog {
    pub(crate) fn new(device: &model::Device) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();

        // Populate free slot list
        let mut free_slots = Vec::new();
        let channels = device.channels();
        for i in 1..=7u32 {
            let used = channels.iter().any(|c| c.index() == i && c.is_active());
            if !used {
                imp.slot_model.append(&format!("Slot {}", i));
                free_slots.push(i);
            }
        }

        if free_slots.is_empty() {
            imp.create_button.set_sensitive(false);
            imp.name_entry.set_sensitive(false);
            // Replace info label text
        }

        imp.free_slots.replace(free_slots);
        imp.device.replace(Some(device.clone()));

        obj
    }

    fn on_create(&self) {
        let imp = self.imp();

        let name = imp.name_entry.text().to_string();
        if name.is_empty() {
            return;
        }

        let free_slots = imp.free_slots.borrow();
        let slot_selection = imp.slot_row.selected() as usize;
        if slot_selection >= free_slots.len() {
            return;
        }
        let index = free_slots[slot_selection];

        // Determine PSK
        let psk = match imp.psk_type_row.selected() {
            0 => {
                // Default key (1-byte shorthand = 1)
                vec![1u8]
            }
            1 => {
                // Simple key (1-byte shorthand = 2..10, use slot+1)
                vec![(index as u8) + 1]
            }
            2 => {
                // Random AES256
                let mut key = vec![0u8; 32];
                for byte in &mut key {
                    *byte = rand_byte();
                }
                key
            }
            3 => {
                // No encryption
                vec![]
            }
            _ => vec![1u8],
        };

        if let Some(device) = imp.device.borrow().as_ref() {
            device.create_channel(index, &name, psk);
        }

        self.close();
    }
}

/// Simple random byte using std (no extra deps)
fn rand_byte() -> u8 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u8(0);
    h.finish() as u8
}
