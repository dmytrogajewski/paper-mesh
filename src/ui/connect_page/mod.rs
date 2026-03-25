use adw::prelude::ComboRowExt;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;

/// Known USB VID/PID pairs for Meshtastic-compatible devices
const MESHTASTIC_USB_IDS: &[(u16, u16, &str)] = &[
    // Silicon Labs CP210x — T-Beam, T-LoRa, Heltec v2, many others
    (0x10C4, 0xEA60, "CP210x"),
    // WCH CH9102 — Heltec v3, LILYGO T3-S3
    (0x1A86, 0x55D4, "CH9102"),
    // WCH CH340 — cheap NodeMCU clones, some Heltec boards
    (0x1A86, 0x7523, "CH340"),
    // Espressif ESP32-S3 native USB — RAK4631, T-Beam Supreme, Station G2
    (0x303A, 0x1001, "ESP32-S3"),
    // Espressif ESP32-S2 native USB
    (0x303A, 0x0002, "ESP32-S2"),
    // Espressif USB-JTAG/serial — ESP32-C3/C6
    (0x303A, 0x4001, "ESP32 JTAG"),
    // FTDI FT232R — some custom boards
    (0x0403, 0x6001, "FT232R"),
    // FTDI FT232H
    (0x0403, 0x6014, "FT232H"),
    // nRF52840 Dongle (Nordic Semiconductor) — RAK4631 via USB
    (0x239A, 0x8029, "nRF52840"),
    // Adafruit nRF52840
    (0x239A, 0x0029, "nRF52840"),
];

/// Discovered serial port with metadata
#[derive(Debug, Clone)]
struct DiscoveredPort {
    path: String,
    label: String,
    is_meshtastic: bool,
}

fn discover_ports() -> Vec<DiscoveredPort> {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            log::warn!("Failed to enumerate serial ports: {e}");
            return vec![];
        }
    };

    let mut result = Vec::new();

    for port in ports {
        match &port.port_type {
            serialport::SerialPortType::UsbPort(usb) => {
                let vid = usb.vid;
                let pid = usb.pid;
                let product = usb
                    .product
                    .as_deref()
                    .unwrap_or("Unknown");
                let manufacturer = usb
                    .manufacturer
                    .as_deref()
                    .unwrap_or("");

                let matched_chip = MESHTASTIC_USB_IDS
                    .iter()
                    .find(|(v, p, _)| *v == vid && *p == pid)
                    .map(|(_, _, name)| *name);

                let is_meshtastic = matched_chip.is_some();

                let label = if let Some(chip) = matched_chip {
                    if manufacturer.is_empty() {
                        format!("{} — {} [{}]", port.port_name, product, chip)
                    } else {
                        format!(
                            "{} — {} {} [{}]",
                            port.port_name, manufacturer, product, chip
                        )
                    }
                } else {
                    format!(
                        "{} — {} ({:04X}:{:04X})",
                        port.port_name, product, vid, pid
                    )
                };

                result.push(DiscoveredPort {
                    path: port.port_name,
                    label,
                    is_meshtastic,
                });
            }
            serialport::SerialPortType::PciPort => {
                result.push(DiscoveredPort {
                    path: port.port_name.clone(),
                    label: format!("{} — PCI serial", port.port_name),
                    is_meshtastic: false,
                });
            }
            _ => {
                result.push(DiscoveredPort {
                    path: port.port_name.clone(),
                    label: port.port_name.clone(),
                    is_meshtastic: false,
                });
            }
        }
    }

    // Sort: Meshtastic-compatible devices first
    result.sort_by(|a, b| b.is_meshtastic.cmp(&a.is_meshtastic));

    result
}

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-mesh/ui/connect_page/connect_page.ui")]
    pub(crate) struct ConnectPage {
        #[template_child]
        pub(super) connection_type_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) serial_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(super) serial_port_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) serial_port_model: TemplateChild<gtk::StringList>,
        #[template_child]
        pub(super) tcp_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(super) tcp_address_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) connect_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) spinner: TemplateChild<gtk::Spinner>,
        pub(super) device: RefCell<Option<model::Device>>,
        pub(super) discovered_ports: RefCell<Vec<DiscoveredPort>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ConnectPage {
        const NAME: &'static str = "PaplConnectPage";
        type Type = super::ConnectPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("connect-page.connect", None, move |obj, _, _| {
                obj.on_connect();
            });

            klass.install_action("connect-page.refresh-ports", None, move |obj, _, _| {
                obj.refresh_serial_ports();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ConnectPage {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Toggle serial/TCP groups based on connection type
            self.connection_type_row.connect_selected_notify(
                clone!(@weak obj => move |row| {
                    let imp = obj.imp();
                    let is_serial = row.selected() == 0;
                    imp.serial_group.set_visible(is_serial);
                    imp.tcp_group.set_visible(!is_serial);
                }),
            );

            // Auto-scan serial ports on startup
            obj.refresh_serial_ports();
        }
    }

    impl WidgetImpl for ConnectPage {}
    impl BinImpl for ConnectPage {}
}

glib::wrapper! {
    pub(crate) struct ConnectPage(ObjectSubclass<imp::ConnectPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl ConnectPage {
    pub(crate) fn set_device(&self, device: &model::Device) {
        let imp = self.imp();

        device.connect_notify_local(
            Some("state"),
            clone!(@weak self as obj => move |device, _| {
                let imp = obj.imp();
                match device.state() {
                    model::DeviceState::Connecting => {
                        imp.spinner.set_spinning(true);
                        imp.connect_button.set_sensitive(false);
                    }
                    model::DeviceState::Error => {
                        imp.spinner.set_spinning(false);
                        imp.status_label.set_label(&device.error_message());
                        imp.connect_button.set_sensitive(true);
                    }
                    model::DeviceState::Disconnected => {
                        imp.spinner.set_spinning(false);
                        imp.status_label.set_label("");
                        imp.connect_button.set_sensitive(true);
                    }
                    _ => {}
                }
            }),
        );

        // Show detailed status during connection
        device.connect_notify_local(
            Some("status-message"),
            clone!(@weak self as obj => move |device, _| {
                let imp = obj.imp();
                if device.state() == model::DeviceState::Connecting {
                    imp.status_label.set_label(&device.status_message());
                }
            }),
        );

        imp.device.replace(Some(device.clone()));
    }

    fn refresh_serial_ports(&self) {
        let imp = self.imp();

        // Clear existing entries
        let model = &imp.serial_port_model;
        while model.n_items() > 0 {
            model.remove(0);
        }

        // Discover ports in a background thread
        let (tx, rx) = async_channel::bounded::<Vec<DiscoveredPort>>(1);

        std::thread::spawn(move || {
            let ports = discover_ports();
            let _ = tx.send_blocking(ports);
        });

        let page = self.downgrade();
        crate::utils::spawn(async move {
            if let Ok(ports) = rx.recv().await {
                let Some(page) = page.upgrade() else {
                    return;
                };
                let imp = page.imp();
                let model = &imp.serial_port_model;

                let mesh_count = ports.iter().filter(|p| p.is_meshtastic).count();

                if ports.is_empty() {
                    model.append("No devices found");
                    imp.discovered_ports.replace(vec![]);
                    imp.status_label.set_label(
                        "No serial devices found. Plug in a Meshtastic radio and click refresh.",
                    );
                } else {
                    for port in &ports {
                        model.append(&port.label);
                    }

                    if mesh_count > 0 {
                        imp.status_label.set_label(&format!(
                            "Found {} Meshtastic device{}",
                            mesh_count,
                            if mesh_count == 1 { "" } else { "s" }
                        ));
                    } else {
                        imp.status_label.set_label(
                            "No recognized Meshtastic devices. Showing all serial ports.",
                        );
                    }

                    imp.discovered_ports.replace(ports);
                }
            }
        });
    }

    fn on_connect(&self) {
        let imp = self.imp();
        let device = imp.device.borrow();
        let Some(device) = device.as_ref() else {
            return;
        };

        let conn_type = imp.connection_type_row.selected();
        let method = if conn_type == 0 {
            // Serial
            let ports = imp.discovered_ports.borrow();
            let selected = imp.serial_port_row.selected() as usize;
            if ports.is_empty() || selected >= ports.len() {
                imp.status_label.set_label("No serial device selected");
                return;
            }
            model::ConnectionMethod::Serial(ports[selected].path.clone())
        } else {
            // TCP
            let address = imp.tcp_address_entry.text().to_string();
            if address.is_empty() {
                imp.status_label.set_label("Please enter a TCP address");
                return;
            }
            model::ConnectionMethod::Tcp(address)
        };

        device.connect(method);
    }
}
