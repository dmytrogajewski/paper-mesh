use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/window.ui")]
    pub(crate) struct Window {
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(super) connect_page: TemplateChild<ui::ConnectPage>,
        #[template_child]
        pub(super) session: TemplateChild<ui::Session>,
        pub(super) device: OnceCell<model::Device>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "PaplWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let device = model::Device::default();
            self.device.set(device.clone()).unwrap();

            let obj = self.obj();

            // When device state changes, switch between connect page and session
            device.connect_notify_local(
                Some("state"),
                clone!(@weak obj => move |device, _| {
                    match device.state() {
                        model::DeviceState::Connected => {
                            obj.imp().session.set_device(device);
                            obj.imp().stack.set_visible_child_name("session");
                        }
                        model::DeviceState::Disconnected => {
                            obj.imp().session.reset();
                            obj.imp().stack.set_visible_child_name("connect");
                        }
                        _ => {}
                    }
                }),
            );

            // Wire up connect page
            self.connect_page.set_device(&device);
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub(crate) struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Accessible;
}

impl Window {
    pub(crate) fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    pub(crate) fn device(&self) -> &model::Device {
        self.imp().device.get().unwrap()
    }
}
