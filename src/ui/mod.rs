mod connect_page;
mod components;
mod session;
mod window;

use gtk::glib::prelude::*;

pub(crate) use self::connect_page::ConnectPage;
pub(crate) use self::components::MessageEntry;
pub(crate) use self::session::AddChannelDialog;
pub(crate) use self::session::Content;
pub(crate) use self::session::MapView;
pub(crate) use self::session::MessageRow;
pub(crate) use self::session::NodeRow;
pub(crate) use self::session::Session;
pub(crate) use self::session::Sidebar;
pub(crate) use self::session::SidebarRow;
pub(crate) use self::window::Window;

pub(crate) fn init() {
    AddChannelDialog::static_type();
    ConnectPage::static_type();
    Content::static_type();
    MapView::static_type();
    MessageEntry::static_type();
    MessageRow::static_type();
    NodeRow::static_type();
    Session::static_type();
    Sidebar::static_type();
    SidebarRow::static_type();
    Window::static_type();
}
