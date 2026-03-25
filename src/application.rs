use std::cell::OnceCell;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::config;
use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Application {
        pub(super) window: OnceCell<glib::WeakRef<ui::Window>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "PaplApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            log::debug!("GtkApplication<Application>::activate");

            let obj = self.obj();

            if let Some(window) = self.window.get() {
                window.upgrade().unwrap().present();
                return;
            }

            let window = ui::Window::new(&obj.upcast_ref::<adw::Application>());
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            obj.main_window().present();
        }

        fn startup(&self) {
            log::debug!("GtkApplication<Application>::startup");

            log::info!("Paper Mesh ({})", config::APP_ID);
            log::info!("Version: {} ({})", config::VERSION, config::PROFILE);
            log::info!("Datadir: {}", config::PKGDATADIR);

            self.parent_startup();

            let obj = self.obj();

            gtk::Window::set_default_icon_name(config::APP_ID);

            obj.setup_gactions();
            obj.setup_accels();
            obj.load_color_scheme();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub(crate) struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub(crate) fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/app/drey/paper-mesh/")
            .build()
    }

    fn main_window(&self) -> ui::Window {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            app.main_window().close();
            app.quit();
        }));
        self.add_action(&action_quit);

        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action_about);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn load_color_scheme(&self) {
        let style_manager = adw::StyleManager::default();
        let settings = gio::Settings::new(config::APP_ID);
        match settings.string("color-scheme").as_ref() {
            "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
            "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
            _ => style_manager.set_color_scheme(adw::ColorScheme::PreferLight),
        }
    }

    fn show_about_dialog(&self) {
        let about = adw::AboutWindow::builder()
            .transient_for(&self.main_window())
            .application_name("Paper Mesh")
            .application_icon(config::APP_ID)
            .version(config::VERSION)
            .website("https://github.com/dmytrogajewski/paper-mesh")
            .issue_url("https://github.com/dmytrogajewski/paper-mesh/issues")
            .developer_name(gettext("Paper Mesh developers"))
            .license_type(gtk::License::Gpl30)
            .build();

        about.present();
    }
}
