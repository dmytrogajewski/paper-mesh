# Paper Mesh Roadmap

## Phase 1 — Core Features

### Direct Messages
Send text messages to a specific node instead of broadcasting to the whole channel. Add a node picker UI when composing a message, and separate DM conversations in the sidebar.

### Node List Panel
Dedicated view showing all discovered mesh nodes with:
- Long name, short name, node number
- Hardware model
- Battery level (with icon)
- Last heard timestamp
- GPS coordinates (lat/lon/altitude)
- Distance from local node (calculated from GPS)
- Online/offline indicator

### Position Map
Display node positions on an interactive map using libshumate (already available as a system dependency). Show each node as a labeled marker. Update positions in real time as GPS packets arrive.

### Waypoints
Send and receive shared map markers (Meshtastic Waypoint packets). Display waypoints on the map with name, description, and icon. Allow creating new waypoints by long-pressing the map.

---

## Phase 2 — Quality of Life

### Channel QR Codes / URL Sharing
Meshtastic encodes channel configuration as shareable URLs (`meshtastic://...`). Generate QR codes and copyable URLs from channel settings so users can easily invite others to a channel.

### Telemetry Graphs
Display device telemetry over time for each node:
- Battery level
- Voltage
- Channel utilization
- Air utilization TX

Store telemetry history locally and render as simple line charts.

### Range Test
Built-in range test tool. Send sequential numbered packets and measure:
- Round-trip time
- Packet loss rate
- SNR/RSSI at each distance
- Max range achieved

### Unread Indicators
Show badge counts on channels in the sidebar for messages received while the user is viewing a different channel. Clear the badge when the channel is selected.

### Desktop Notifications
Send GNOME desktop notifications (via `GNotification`) for incoming messages when the app is in the background or the message is on a non-active channel.

### Node Online/Offline Tracking
Track node heartbeats and mark nodes as offline after a configurable timeout (default: 2 hours since last heard). Show visual distinction between online and offline nodes in the node list.

---

## Phase 3 — Device Configuration

### Device Settings UI
Configure radio parameters from the app instead of requiring the CLI:
- Region (US, EU, etc.)
- Modem preset (Long Fast, Long Slow, Short Fast, etc.)
- Hop limit
- TX power
- Device role (Client, Router, Repeater, etc.)
- Device name (long name / short name)

### Module Configuration
Enable/disable and configure Meshtastic modules:
- Store & Forward — buffer messages for offline nodes
- Range Test — automated range testing
- Telemetry — configure reporting intervals
- MQTT — uplink/downlink configuration
- Serial module, External notification, Canned messages, etc.

### Firmware Update
Over-the-air (OTA) firmware update support. Check for new firmware versions, download, and flash to the connected device.

---

## Phase 4 — Advanced Features

### Multiple Device Connections
Connect to several Meshtastic radios simultaneously (e.g. different frequency bands or regions). Show a device switcher and aggregate messages across connections.

### Message Delivery Receipts
Track ACK status for sent messages. Show sent/delivered/failed indicators on outgoing messages using the Meshtastic want_ack mechanism.

### Canned Messages
Quick-reply presets common in Meshtastic field use. Allow users to define a list of canned messages and send them with one tap. Sync with the device's canned message module if configured.

### File / Binary Transfer
Chunked data transfer over mesh. Very slow due to LoRa bandwidth constraints but useful for small files (GPS tracks, short voice clips). Requires a chunking protocol on top of Meshtastic's 228-byte text limit.

### MQTT Bridge Status
Show whether the connected device has an active MQTT uplink. Display MQTT connection status, topic, and allow toggling uplink/downlink per channel.
