#![allow(clippy::format_push_string)]

mod application;
mod ui;
#[rustfmt::skip]
#[allow(clippy::all)]
mod config;
mod i18n;
mod model;
mod types;
mod utils;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;

use gettextrs::gettext;
use gettextrs::LocaleCategory;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use temp_dir::TempDir;

use self::application::Application;

pub(crate) static APPLICATION_OPTS: OnceLock<ApplicationOptions> = OnceLock::new();
pub(crate) static TEMP_DIR: OnceLock<PathBuf> = OnceLock::new();

fn main() -> glib::ExitCode {
    let app = setup_cli(Application::default());

    app.connect_handle_local_options(|_, dict| {
        if dict.contains("version") {
            println!("paper-mesh {}", config::VERSION);
            1
        } else {
            adw::init().expect("Failed to init GTK/libadwaita");
            ui::init();

            gettextrs::setlocale(LocaleCategory::LcAll, "");
            gettextrs::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
                .expect("Unable to bind the text domain");
            gettextrs::textdomain(config::GETTEXT_PACKAGE)
                .expect("Unable to switch to the text domain");

            glib::set_application_name("Paper Mesh");

            gio::resources_register(
                &gio::Resource::load(config::RESOURCES_FILE)
                    .expect("Could not load gresource file"),
            );
            gio::resources_register(
                &gio::Resource::load(config::UI_RESOURCES_FILE)
                    .expect("Could not load UI gresource file"),
            );

            let log_level = match dict.lookup::<String>("log-level").unwrap() {
                Some(level) => log::Level::from_str(&level).expect("Error on parsing log-level"),
                None => log::Level::Warn,
            };

            let application_opts = ApplicationOptions::default();

            // Suppress noisy meshtastic stream_buffer partial-packet logs
            std::env::set_var(
                "RUST_LOG",
                format!(
                    "{},meshtastic::connections::stream_buffer=off",
                    log_level.as_str()
                ),
            );
            pretty_env_logger::init();

            APPLICATION_OPTS.set(application_opts).unwrap();

            -1
        }
    });

    let temp_dir = TempDir::with_prefix("paper-mesh");
    match &temp_dir {
        Ok(temp_dir) => {
            TEMP_DIR.set(temp_dir.path().to_path_buf()).unwrap();
        }
        Err(e) => {
            log::warn!("Error creating temp directory: {e:?}");
        }
    }

    app.run()
}

/// Global options for the application
#[derive(Debug)]
pub(crate) struct ApplicationOptions {
    pub(crate) data_dir: PathBuf,
}

impl Default for ApplicationOptions {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(glib::user_data_dir().to_str().unwrap()).join("paper-mesh"),
        }
    }
}

fn setup_cli<A: IsA<gio::Application>>(app: A) -> A {
    app.add_main_option(
        "version",
        b'v'.into(),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        &gettext("Prints application version"),
        None,
    );

    app.add_main_option(
        "log-level",
        b'l'.into(),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        &gettext("Specify the minimum log level"),
        Some("error|warn|info|debug|trace"),
    );

    app
}
