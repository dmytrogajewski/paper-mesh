use std::cell::Cell;
use std::cell::RefCell;
use std::fmt;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use meshtastic::Message as ProstMessage;

use super::Channel;
use super::ConnectionMethod;
use super::MeshMessage;
use super::MessageDirection;
use super::NodeList;
use super::Waypoint;
use super::WaypointList;
use super::message_store;
use crate::types::NodeId;
use crate::utils;

/// Messages sent from the background tokio thread to the GTK main loop
#[derive(Debug)]
pub(crate) enum DeviceEvent {
    /// Status text update shown during connection/loading
    Status(String),
    Connected {
        my_node_num: NodeId,
    },
    /// Initial config dump from device is complete
    ConfigComplete,
    Disconnected,
    Error(String),
    NodeInfo {
        num: NodeId,
        long_name: String,
        short_name: String,
        hw_model: String,
    },
    NodeMetrics {
        num: NodeId,
        battery_level: u32,
    },
    NodePosition {
        num: NodeId,
        latitude: f64,
        longitude: f64,
        altitude: i32,
    },
    ChannelInfo {
        index: u32,
        name: String,
        role: u32,
    },
    TextMessage {
        packet_id: u32,
        from: NodeId,
        to: NodeId,
        channel_index: u32,
        text: String,
        rx_time: u32,
        snr: f32,
        rssi: i32,
        hop_start: u32,
        hop_limit: u32,
    },
    WaypointReceived {
        id: u32,
        name: String,
        description: String,
        latitude: f64,
        longitude: f64,
        expire: u32,
        locked_to: u32,
        from_node: NodeId,
    },
}

/// Commands sent from the GTK main loop to the background tokio thread
#[derive(Debug)]
pub(crate) enum DeviceCommand {
    SendText {
        text: String,
        channel_index: u32,
        destination: u32, // 0xFFFFFFFF = broadcast
    },
    CreateChannel {
        index: u32,
        name: String,
        psk: Vec<u8>, // empty = no crypto, 1 byte = simple, 16 = AES128, 32 = AES256
    },
    DeleteChannel {
        index: u32,
    },
    SendWaypoint {
        name: String,
        description: String,
        latitude: f64,
        longitude: f64,
        channel_index: u32,
    },
    Disconnect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "DeviceState")]
pub(crate) enum DeviceState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Device {
        pub(super) state: Cell<DeviceState>,
        pub(super) my_node_num: Cell<NodeId>,
        pub(super) nodes: OnceCell<NodeList>,
        pub(super) waypoints: OnceCell<WaypointList>,
        pub(super) channels: RefCell<Vec<Channel>>,
        pub(super) error_message: RefCell<String>,
        pub(super) status_message: RefCell<String>,
        pub(super) connection_info: RefCell<String>,
        pub(super) config_loading: Cell<bool>,
        pub(super) command_sender:
            RefCell<Option<tokio::sync::mpsc::UnboundedSender<DeviceCommand>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Device {
        const NAME: &'static str = "MeshDevice";
        type Type = super::Device;
    }

    impl ObjectImpl for Device {
        fn constructed(&self) {
            self.parent_constructed();
            self.nodes.set(NodeList::default()).unwrap();
            self.waypoints.set(WaypointList::default()).unwrap();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![glib::subclass::Signal::builder("channels-changed").build()]
            })
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecEnum::builder::<DeviceState>("state")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("error-message")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("status-message")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("connection-info")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("config-loading")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("my-node-num")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "state" => obj.state().to_value(),
                "error-message" => obj.error_message().to_value(),
                "status-message" => obj.status_message().to_value(),
                "connection-info" => obj.connection_info().to_value(),
                "config-loading" => obj.config_loading().to_value(),
                "my-node-num" => obj.my_node_num().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Device(ObjectSubclass<imp::Device>);
}

impl Default for Device {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Device {
    pub(crate) fn state(&self) -> DeviceState {
        self.imp().state.get()
    }

    pub(crate) fn my_node_num(&self) -> NodeId {
        self.imp().my_node_num.get()
    }

    pub(crate) fn nodes(&self) -> &NodeList {
        self.imp().nodes.get().unwrap()
    }

    pub(crate) fn waypoints(&self) -> &WaypointList {
        self.imp().waypoints.get().unwrap()
    }

    pub(crate) fn channels(&self) -> Vec<Channel> {
        self.imp().channels.borrow().clone()
    }

    pub(crate) fn channel(&self, index: u32) -> Option<Channel> {
        self.imp()
            .channels
            .borrow()
            .iter()
            .find(|c| c.index() == index)
            .cloned()
    }

    pub(crate) fn error_message(&self) -> String {
        self.imp().error_message.borrow().clone()
    }

    pub(crate) fn status_message(&self) -> String {
        self.imp().status_message.borrow().clone()
    }

    pub(crate) fn connection_info(&self) -> String {
        self.imp().connection_info.borrow().clone()
    }

    pub(crate) fn config_loading(&self) -> bool {
        self.imp().config_loading.get()
    }

    fn set_state(&self, state: DeviceState) {
        self.imp().state.set(state);
        self.notify("state");
    }

    fn set_status(&self, msg: &str) {
        self.imp().status_message.replace(msg.to_string());
        self.notify("status-message");
    }

    /// Start connecting to a Meshtastic device in a background tokio thread
    pub(crate) fn connect(&self, method: ConnectionMethod) {
        self.set_state(DeviceState::Connecting);
        self.imp().config_loading.set(true);
        self.notify("config-loading");

        // Store connection info for display
        let info = match &method {
            ConnectionMethod::Serial(port) => format!("{} @ 115200 baud", port),
            ConnectionMethod::Tcp(addr) => format!("TCP {}", addr),
        };
        self.imp().connection_info.replace(info);
        self.notify("connection-info");

        self.set_status("Opening connection...");

        let (event_tx, event_rx) = async_channel::unbounded::<DeviceEvent>();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<DeviceCommand>();

        self.imp().command_sender.replace(Some(cmd_tx));

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                if let Err(e) = run_device(method, event_tx.clone(), cmd_rx).await {
                    let _ = event_tx.send(DeviceEvent::Error(format!("{e}"))).await;
                }
            });
        });

        let device = self.downgrade();
        utils::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                let Some(device) = device.upgrade() else {
                    break;
                };
                device.handle_event(event);
            }
        });
    }

    pub(crate) fn disconnect(&self) {
        if let Some(sender) = self.imp().command_sender.borrow().as_ref() {
            let _ = sender.send(DeviceCommand::Disconnect);
        }
        self.set_state(DeviceState::Disconnected);
        self.set_status("");
    }

    pub(crate) fn send_text(&self, text: &str, channel_index: u32, destination: u32) {
        if let Some(sender) = self.imp().command_sender.borrow().as_ref() {
            let _ = sender.send(DeviceCommand::SendText {
                text: text.to_string(),
                channel_index,
                destination,
            });
        }
    }

    pub(crate) fn send_waypoint(
        &self,
        name: &str,
        description: &str,
        lat: f64,
        lon: f64,
        channel_index: u32,
    ) {
        if let Some(sender) = self.imp().command_sender.borrow().as_ref() {
            let _ = sender.send(DeviceCommand::SendWaypoint {
                name: name.to_string(),
                description: description.to_string(),
                latitude: lat,
                longitude: lon,
                channel_index,
            });
        }
    }

    pub(crate) fn create_channel(&self, index: u32, name: &str, psk: Vec<u8>) {
        if let Some(sender) = self.imp().command_sender.borrow().as_ref() {
            let _ = sender.send(DeviceCommand::CreateChannel {
                index,
                name: name.to_string(),
                psk,
            });
        }
        self.set_status(&format!("Creating channel {}...", name));
    }

    pub(crate) fn delete_channel(&self, index: u32) {
        if let Some(sender) = self.imp().command_sender.borrow().as_ref() {
            let _ = sender.send(DeviceCommand::DeleteChannel { index });
        }
        self.set_status("Deleting channel...");
    }

    /// Returns the first free channel slot (1-7), or None if all are taken
    pub(crate) fn next_free_channel_index(&self) -> Option<u32> {
        let channels = self.imp().channels.borrow();
        for i in 1..=7u32 {
            let used = channels.iter().any(|c| c.index() == i && c.is_active());
            if !used {
                return Some(i);
            }
        }
        None
    }

    fn handle_event(&self, event: DeviceEvent) {
        match event {
            DeviceEvent::Status(msg) => {
                self.set_status(&msg);
            }
            DeviceEvent::Connected { my_node_num } => {
                self.imp().my_node_num.set(my_node_num);
                self.set_state(DeviceState::Connected);
                self.notify("my-node-num");
                self.set_status("Receiving configuration...");
                log::info!("Connected to device, my node: !{:08x}", my_node_num);
            }
            DeviceEvent::ConfigComplete => {
                self.imp().config_loading.set(false);
                self.notify("config-loading");
                let nodes = self.nodes().len();
                let channels = self
                    .channels()
                    .iter()
                    .filter(|c| c.is_active())
                    .count();
                self.set_status(&format!(
                    "Ready — {} channel{}, {} node{}",
                    channels,
                    if channels == 1 { "" } else { "s" },
                    nodes,
                    if nodes == 1 { "" } else { "s" },
                ));
            }
            DeviceEvent::Disconnected => {
                self.set_state(DeviceState::Disconnected);
                self.set_status("Disconnected");
            }
            DeviceEvent::Error(msg) => {
                log::error!("Device error: {}", msg);
                self.imp().error_message.replace(msg.clone());
                self.set_state(DeviceState::Error);
                self.set_status(&format!("Error: {}", msg));
                self.notify("error-message");
                self.imp().config_loading.set(false);
                self.notify("config-loading");
            }
            DeviceEvent::NodeInfo {
                num,
                long_name,
                short_name,
                hw_model,
            } => {
                let node = self.nodes().add_or_update(num);
                node.set_long_name(&long_name);
                node.set_short_name(&short_name);
                node.set_hw_model(&hw_model);
                node.set_is_online(true);
                if self.config_loading() {
                    self.set_status(&format!(
                        "Loading nodes... ({})",
                        self.nodes().len()
                    ));
                }
            }
            DeviceEvent::NodeMetrics { num, battery_level } => {
                let node = self.nodes().add_or_update(num);
                node.set_battery_level(battery_level);
            }
            DeviceEvent::NodePosition {
                num,
                latitude,
                longitude,
                altitude,
            } => {
                let node = self.nodes().add_or_update(num);
                node.set_position(latitude, longitude, altitude);
            }
            DeviceEvent::ChannelInfo { index, name, role } => {
                let channel = {
                    let channels = self.imp().channels.borrow();
                    channels.iter().find(|c| c.index() == index).cloned()
                };
                if let Some(channel) = channel {
                    channel.set_name(&name);
                    channel.set_role(role);
                } else {
                    let channel = Channel::new(index);
                    channel.set_name(&name);
                    channel.set_role(role);
                    // Load persisted messages for this channel
                    let stored = message_store::load_messages(index);
                    for sm in &stored {
                        channel.messages().append(sm.to_mesh_message());
                    }
                    self.imp().channels.borrow_mut().push(channel);
                    self.emit_by_name::<()>("channels-changed", &[]);
                }
                if self.config_loading() {
                    let active = self
                        .channels()
                        .iter()
                        .filter(|c| c.is_active())
                        .count();
                    self.set_status(&format!("Loading channels... ({})", active));
                }
            }
            DeviceEvent::TextMessage {
                packet_id,
                from,
                to,
                channel_index,
                text,
                rx_time,
                snr,
                rssi,
                hop_start,
                hop_limit,
            } => {
                let direction = if from == self.my_node_num() {
                    MessageDirection::Outgoing
                } else {
                    MessageDirection::Incoming
                };

                let message = MeshMessage::new(
                    packet_id,
                    from,
                    to,
                    channel_index,
                    &text,
                    rx_time,
                    direction,
                );
                message.set_radio_info(snr, rssi, hop_start, hop_limit);

                if let Some(node) = self.nodes().find_by_num(from) {
                    message.set_sender_name(&node.display_name());
                } else {
                    message.set_sender_name(&format!("!{:08x}", from));
                }

                // Persist to disk
                message_store::append_message(channel_index, &message);

                if let Some(channel) = self.channel(channel_index) {
                    channel.messages().append(message);
                } else {
                    let channel = Channel::new(channel_index);
                    channel.set_role(1);
                    // Load any prior persisted messages first
                    let stored = message_store::load_messages(channel_index);
                    for sm in &stored {
                        if sm.packet_id != packet_id {
                            channel.messages().append(sm.to_mesh_message());
                        }
                    }
                    channel.messages().append(message);
                    self.imp().channels.borrow_mut().push(channel);
                    self.emit_by_name::<()>("channels-changed", &[]);
                }
            }
            DeviceEvent::WaypointReceived {
                id,
                name,
                description,
                latitude,
                longitude,
                expire,
                locked_to,
                from_node,
            } => {
                let waypoint = Waypoint::new(
                    id, &name, &description, latitude, longitude, expire, locked_to, from_node,
                );
                self.waypoints().add_or_update(waypoint);
                log::info!(
                    "Waypoint received: {} ({}, {}) from !{:08x}",
                    name, latitude, longitude, from_node
                );
            }
        }
    }
}

// Simple error type for the PacketRouter trait
#[derive(Debug)]
struct RouterError;

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "router error")
    }
}

impl std::error::Error for RouterError {}

/// Process a single FromRadio payload into DeviceEvents
async fn handle_payload(
    event_tx: &async_channel::Sender<DeviceEvent>,
    payload: &meshtastic::protobufs::from_radio::PayloadVariant,
    my_node_num: NodeId,
) -> anyhow::Result<()> {
    use meshtastic::protobufs::from_radio::PayloadVariant;

    match payload {
        PayloadVariant::NodeInfo(node_info) => {
            let num = node_info.num;
            let (long_name, short_name, hw_model) = if let Some(user) = &node_info.user {
                let hw = format!("{:?}", user.hw_model());
                (user.long_name.clone(), user.short_name.clone(), hw)
            } else {
                (String::new(), String::new(), String::new())
            };

            event_tx
                .send(DeviceEvent::NodeInfo {
                    num,
                    long_name,
                    short_name,
                    hw_model,
                })
                .await?;

            if let Some(position) = &node_info.position {
                let lat = position.latitude_i.unwrap_or(0) as f64 * 1e-7;
                let lon = position.longitude_i.unwrap_or(0) as f64 * 1e-7;
                let alt = position.altitude.unwrap_or(0);
                event_tx
                    .send(DeviceEvent::NodePosition {
                        num,
                        latitude: lat,
                        longitude: lon,
                        altitude: alt,
                    })
                    .await?;
            }

            if let Some(metrics) = &node_info.device_metrics {
                event_tx
                    .send(DeviceEvent::NodeMetrics {
                        num,
                        battery_level: metrics.battery_level.unwrap_or(0),
                    })
                    .await?;
            }
        }
        PayloadVariant::Channel(channel) => {
            let index = channel.index as u32;
            let (name, role) = if let Some(settings) = &channel.settings {
                (settings.name.clone(), channel.role)
            } else {
                (String::new(), channel.role)
            };
            event_tx
                .send(DeviceEvent::ChannelInfo {
                    index,
                    name,
                    role: role as u32,
                })
                .await?;
        }
        PayloadVariant::Packet(mesh_packet) => {
            let from = mesh_packet.from;
            let to = mesh_packet.to;
            let channel = mesh_packet.channel;
            let rx_time = mesh_packet.rx_time;
            let snr = mesh_packet.rx_snr;
            let rssi = mesh_packet.rx_rssi;
            let hop_start = mesh_packet.hop_start;
            let hop_limit = mesh_packet.hop_limit;
            let packet_id = mesh_packet.id;

            if let Some(meshtastic::protobufs::mesh_packet::PayloadVariant::Decoded(data)) =
                &mesh_packet.payload_variant
            {
                if data.portnum() == meshtastic::protobufs::PortNum::TextMessageApp {
                    if let Ok(text) = String::from_utf8(data.payload.clone()) {
                        event_tx
                            .send(DeviceEvent::TextMessage {
                                packet_id,
                                from,
                                to,
                                channel_index: channel,
                                text,
                                rx_time,
                                snr,
                                rssi,
                                hop_start,
                                hop_limit,
                            })
                            .await?;
                    }
                }
                if data.portnum() == meshtastic::protobufs::PortNum::PositionApp {
                    if let Ok(position) =
                        meshtastic::protobufs::Position::decode(data.payload.as_slice())
                    {
                        let lat = position.latitude_i.unwrap_or(0) as f64 * 1e-7;
                        let lon = position.longitude_i.unwrap_or(0) as f64 * 1e-7;
                        event_tx
                            .send(DeviceEvent::NodePosition {
                                num: from,
                                latitude: lat,
                                longitude: lon,
                                altitude: position.altitude.unwrap_or(0),
                            })
                            .await?;
                    }
                }
                if data.portnum() == meshtastic::protobufs::PortNum::TelemetryApp {
                    if let Ok(telemetry) =
                        meshtastic::protobufs::Telemetry::decode(data.payload.as_slice())
                    {
                        if let Some(
                            meshtastic::protobufs::telemetry::Variant::DeviceMetrics(metrics),
                        ) = telemetry.variant
                        {
                            event_tx
                                .send(DeviceEvent::NodeMetrics {
                                    num: from,
                                    battery_level: metrics.battery_level.unwrap_or(0),
                                })
                                .await?;
                        }
                    }
                }
                if data.portnum() == meshtastic::protobufs::PortNum::NodeinfoApp {
                    if let Ok(user) =
                        meshtastic::protobufs::User::decode(data.payload.as_slice())
                    {
                        let hw = format!("{:?}", user.hw_model());
                        event_tx
                            .send(DeviceEvent::NodeInfo {
                                num: from,
                                long_name: user.long_name,
                                short_name: user.short_name,
                                hw_model: hw,
                            })
                            .await?;
                    }
                }
                if data.portnum() == meshtastic::protobufs::PortNum::WaypointApp {
                    if let Ok(wp) =
                        meshtastic::protobufs::Waypoint::decode(data.payload.as_slice())
                    {
                        event_tx
                            .send(DeviceEvent::WaypointReceived {
                                id: wp.id,
                                name: wp.name,
                                description: wp.description,
                                latitude: wp.latitude_i.unwrap_or(0) as f64 * 1e-7,
                                longitude: wp.longitude_i.unwrap_or(0) as f64 * 1e-7,
                                expire: wp.expire,
                                locked_to: wp.locked_to,
                                from_node: from,
                            })
                            .await?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Background task that runs the Meshtastic connection
async fn run_device(
    method: ConnectionMethod,
    event_tx: async_channel::Sender<DeviceEvent>,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<DeviceCommand>,
) -> anyhow::Result<()> {
    use meshtastic::api::StreamApi;
    use meshtastic::utils;

    let stream_api = StreamApi::new();

    event_tx
        .send(DeviceEvent::Status("Opening serial port...".into()))
        .await?;

    let (mut decoded_listener, stream_api) = match method {
        ConnectionMethod::Serial(port) => {
            event_tx
                .send(DeviceEvent::Status(format!(
                    "Connecting to {}...",
                    port
                )))
                .await?;
            let serial_stream = utils::stream::build_serial_stream(port, None, None, None)?;
            stream_api.connect(serial_stream).await
        }
        ConnectionMethod::Tcp(address) => {
            event_tx
                .send(DeviceEvent::Status(format!(
                    "Connecting to {}...",
                    address
                )))
                .await?;
            let tcp_stream = utils::stream::build_tcp_stream(address).await?;
            stream_api.connect(tcp_stream).await
        }
    };

    event_tx
        .send(DeviceEvent::Status(
            "Connected. Requesting configuration...".into(),
        ))
        .await?;

    let config_id = utils::generate_rand_id();
    let mut stream_api = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        stream_api.configure(config_id),
    )
    .await
    {
        Ok(Ok(api)) => api,
        Ok(Err(e)) => {
            return Err(anyhow::anyhow!("Configuration failed: {e}"));
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Configuration timed out after 30s. Try disconnecting and reconnecting the device."
            ));
        }
    };

    // configure() just sends WantConfigId — the actual config data
    // (MyInfo, NodeInfo, Channel, ConfigCompleteId) arrives via decoded_listener.
    // Drain what we can immediately to get MyInfo early.

    let mut my_node_num: NodeId = 0;
    let mut got_config_complete = false;

    // First, drain initial config burst with a short timeout per packet.
    // The device sends config data rapidly after WantConfigId.
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            decoded_listener.recv(),
        ).await {
            Ok(Some(from_radio)) => {
                if let Some(payload) = from_radio.payload_variant {
                    use meshtastic::protobufs::from_radio::PayloadVariant;
                    match &payload {
                        PayloadVariant::MyInfo(info) => {
                            my_node_num = info.my_node_num;
                            event_tx.send(DeviceEvent::Connected {
                                my_node_num: info.my_node_num,
                            }).await?;
                        }
                        PayloadVariant::ConfigCompleteId(_) => {
                            got_config_complete = true;
                            event_tx.send(DeviceEvent::ConfigComplete).await?;
                        }
                        _ => {}
                    }
                    // Process config data through the normal handler
                    handle_payload(&event_tx, &payload, my_node_num).await?;

                    if got_config_complete {
                        break;
                    }
                }
            }
            Ok(None) => {
                event_tx.send(DeviceEvent::Disconnected).await?;
                return Ok(());
            }
            Err(_) => {
                // Timeout — no more config packets
                log::warn!("Config drain timed out, proceeding anyway");
                if my_node_num == 0 {
                    event_tx.send(DeviceEvent::Error(
                        "Device did not send node info. Try reconnecting.".into()
                    )).await?;
                    return Ok(());
                }
                // Send config complete even if we didn't get the explicit packet
                if !got_config_complete {
                    event_tx.send(DeviceEvent::ConfigComplete).await?;
                }
                break;
            }
        }
    }

    // Main event loop — handle ongoing mesh packets and commands
    loop {
        tokio::select! {
            decoded = decoded_listener.recv() => {
                let Some(from_radio) = decoded else {
                    event_tx.send(DeviceEvent::Disconnected).await?;
                    break;
                };
                if let Some(ref payload) = from_radio.payload_variant {
                    handle_payload(&event_tx, payload, my_node_num).await?;
                }
            }
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break; };
                match cmd {
                    DeviceCommand::SendText { text, channel_index, destination } => {
                        use meshtastic::packet::PacketDestination;

                        let dest = if destination == 0xFFFFFFFF {
                            PacketDestination::Broadcast
                        } else {
                            PacketDestination::Node(destination.into())
                        };

                        let channel = meshtastic::types::MeshChannel::new(channel_index)?;
                        let mut router = NoOpRouter;

                        if let Err(e) = stream_api
                            .send_text(&mut router, text.clone(), dest, true, channel)
                            .await
                        {
                            log::error!("Failed to send text: {e}");
                            event_tx.send(DeviceEvent::Error(format!("Send failed: {e}"))).await?;
                        } else {
                            let rx_time = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as u32;
                            event_tx.send(DeviceEvent::TextMessage {
                                packet_id: meshtastic::utils::generate_rand_id(),
                                from: my_node_num,
                                to: destination,
                                channel_index,
                                text,
                                rx_time,
                                snr: 0.0,
                                rssi: 0,
                                hop_start: 0,
                                hop_limit: 0,
                            }).await?;
                        }
                    }
                    DeviceCommand::CreateChannel { index, name, psk } => {
                        let mut router = NoOpRouter;

                        #[allow(deprecated)]
                        let channel_config = meshtastic::protobufs::Channel {
                            index: index as i32,
                            settings: Some(meshtastic::protobufs::ChannelSettings {
                                channel_num: 0,
                                psk,
                                name: name.clone(),
                                id: 0,
                                uplink_enabled: false,
                                downlink_enabled: false,
                                module_settings: None,
                            }),
                            role: meshtastic::protobufs::channel::Role::Secondary as i32,
                        };

                        event_tx.send(DeviceEvent::Status(
                            format!("Creating channel '{}'...", name)
                        )).await?;

                        stream_api.start_config_transaction().await?;
                        if let Err(e) = stream_api
                            .update_channel_config(&mut router, channel_config)
                            .await
                        {
                            log::error!("Failed to create channel: {e}");
                            event_tx.send(DeviceEvent::Error(format!("Create channel failed: {e}"))).await?;
                        } else {
                            stream_api.commit_config_transaction().await?;
                            event_tx.send(DeviceEvent::Status(
                                format!("Channel '{}' created. Device restarting...", name)
                            )).await?;
                            // Device will restart, which closes the connection
                        }
                    }
                    DeviceCommand::DeleteChannel { index } => {
                        let mut router = NoOpRouter;

                        #[allow(deprecated)]
                        let channel_config = meshtastic::protobufs::Channel {
                            index: index as i32,
                            settings: Some(meshtastic::protobufs::ChannelSettings {
                                channel_num: 0,
                                psk: vec![],
                                name: String::new(),
                                id: 0,
                                uplink_enabled: false,
                                downlink_enabled: false,
                                module_settings: None,
                            }),
                            role: meshtastic::protobufs::channel::Role::Disabled as i32,
                        };

                        event_tx.send(DeviceEvent::Status("Deleting channel...".into())).await?;

                        stream_api.start_config_transaction().await?;
                        if let Err(e) = stream_api
                            .update_channel_config(&mut router, channel_config)
                            .await
                        {
                            log::error!("Failed to delete channel: {e}");
                            event_tx.send(DeviceEvent::Error(format!("Delete channel failed: {e}"))).await?;
                        } else {
                            stream_api.commit_config_transaction().await?;
                            event_tx.send(DeviceEvent::Status(
                                "Channel deleted. Device restarting...".into()
                            )).await?;
                        }
                    }
                    DeviceCommand::SendWaypoint { name, description, latitude, longitude, channel_index } => {
                        let mut router = NoOpRouter;
                        let waypoint = meshtastic::protobufs::Waypoint {
                            id: meshtastic::utils::generate_rand_id(),
                            latitude_i: Some((latitude * 1e7) as i32),
                            longitude_i: Some((longitude * 1e7) as i32),
                            expire: 0,
                            locked_to: 0,
                            name: name.clone(),
                            description: description.clone(),
                            icon: 0,
                        };
                        let channel = meshtastic::types::MeshChannel::new(channel_index)?;
                        if let Err(e) = stream_api
                            .send_waypoint(&mut router, waypoint, meshtastic::packet::PacketDestination::Broadcast, true, channel)
                            .await
                        {
                            log::error!("Failed to send waypoint: {e}");
                            event_tx.send(DeviceEvent::Error(format!("Send waypoint failed: {e}"))).await?;
                        } else {
                            event_tx.send(DeviceEvent::WaypointReceived {
                                id: meshtastic::utils::generate_rand_id(),
                                name,
                                description,
                                latitude,
                                longitude,
                                expire: 0,
                                locked_to: 0,
                                from_node: my_node_num,
                            }).await?;
                        }
                    }
                    DeviceCommand::Disconnect => {
                        let _ = stream_api.disconnect().await;
                        event_tx.send(DeviceEvent::Disconnected).await?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// A no-op packet router that satisfies the meshtastic API requirement
struct NoOpRouter;

impl meshtastic::packet::PacketRouter<(), RouterError> for NoOpRouter {
    fn handle_packet_from_radio(
        &mut self,
        _packet: meshtastic::protobufs::FromRadio,
    ) -> Result<(), RouterError> {
        Ok(())
    }

    fn handle_mesh_packet(
        &mut self,
        _packet: meshtastic::protobufs::MeshPacket,
    ) -> Result<(), RouterError> {
        Ok(())
    }

    fn source_node_id(&self) -> meshtastic::types::NodeId {
        0.into()
    }
}
