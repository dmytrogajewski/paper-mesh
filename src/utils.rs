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

/// Format a unix timestamp into a human-readable time string (HH:MM local time)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_ampersand() {
        assert_eq!(escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_angle_brackets() {
        assert_eq!(escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape(r#"he said "hi""#), "he said &quot;hi&quot;");
        assert_eq!(escape("it's"), "it&apos;s");
    }

    #[test]
    fn test_escape_no_change() {
        assert_eq!(escape("plain text 123"), "plain text 123");
    }

    #[test]
    fn test_escape_empty() {
        assert_eq!(escape(""), "");
    }

    #[test]
    fn test_freplace_single() {
        let result = freplace("Hello {name}!".into(), &[("name", "Alice")]);
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_freplace_multiple() {
        let result = freplace("{a} and {b}".into(), &[("a", "X"), ("b", "Y")]);
        assert_eq!(result, "X and Y");
    }

    #[test]
    fn test_freplace_no_match() {
        let result = freplace("no vars here".into(), &[("x", "y")]);
        assert_eq!(result, "no vars here");
    }

    #[test]
    fn test_freplace_empty_args() {
        let result = freplace("{x}".into(), &[]);
        assert_eq!(result, "{x}");
    }

    #[test]
    fn test_format_timestamp_valid() {
        let result = format_timestamp(1700000000);
        // Should be a time string like "HH:MM" — just check format
        assert!(result.contains(':'), "Expected HH:MM format, got: {result}");
        assert!(result.len() == 5, "Expected 5 chars, got: {result}");
    }

    #[test]
    fn test_format_timestamp_zero() {
        let result = format_timestamp(0);
        // Epoch 0 is valid — 1970-01-01 00:00 UTC
        assert!(result.contains(':'));
    }
}
