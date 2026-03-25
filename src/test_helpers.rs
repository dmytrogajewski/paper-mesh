#[cfg(test)]
pub(crate) fn init_gtk() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        gtk::init().expect("Failed to init GTK for tests");
    });
}
