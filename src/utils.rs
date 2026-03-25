use std::future::Future;
use std::ops::Deref;
use std::path::PathBuf;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::config;
use crate::APPLICATION_OPTS;
use crate::TEMP_DIR;

#[derive(Debug)]
pub(crate) struct PaperPlaneSettings(gio::Settings);

impl Default for PaperPlaneSettings {
    fn default() -> Self {
        Self(gio::Settings::new(config::APP_ID))
    }
}

impl Deref for PaperPlaneSettings {
    type Target = gio::Settings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Replace variables in the given string with the given dictionary.
pub(crate) fn freplace(s: String, args: &[(&str, &str)]) -> String {
    let mut s = s;
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

/// Returns the Paper Mesh data directory.
pub(crate) fn data_dir() -> &'static PathBuf {
    &APPLICATION_OPTS.get().unwrap().data_dir
}

/// Returns the Paper Mesh temp directory.
pub(crate) fn temp_dir() -> Option<&'static PathBuf> {
    TEMP_DIR.get()
}

/// Spawn a future on the default `MainContext`
pub(crate) fn spawn<F: Future<Output = ()> + 'static>(fut: F) {
    let ctx = glib::MainContext::default();
    ctx.spawn_local(fut);
}

pub(crate) fn show_toast<W: IsA<gtk::Widget>>(widget: &W, title: impl Into<glib::GString>) {
    widget
        .ancestor(adw::ToastOverlay::static_type())
        .unwrap()
        .downcast::<adw::ToastOverlay>()
        .unwrap()
        .add_toast(
            adw::Toast::builder()
                .title(title)
                .timeout(3)
                .priority(adw::ToastPriority::High)
                .build(),
        );
}

pub(crate) fn ancestor<W: IsA<gtk::Widget>, T: IsA<gtk::Widget>>(widget: &W) -> T {
    widget
        .ancestor(T::static_type())
        .and_downcast::<T>()
        .unwrap()
}

/// Format a unix timestamp into a human-readable time string
pub(crate) fn format_timestamp(secs: u32) -> String {
    use chrono::prelude::*;
    let dt = DateTime::from_timestamp(secs as i64, 0);
    match dt {
        Some(dt) => {
            let local: DateTime<Local> = dt.into();
            local.format("%H:%M").to_string()
        }
        None => String::new(),
    }
}
