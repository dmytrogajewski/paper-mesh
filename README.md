# Paper Mesh

A native GNOME desktop client for [Meshtastic](https://meshtastic.org/) mesh networks, built with Rust, GTK4, and libadwaita.

Paper Mesh lets you connect to a Meshtastic LoRa radio and chat with other nodes on the mesh — no phone required.

## Features

- **Device connection** via serial port (USB) or TCP, with automatic detection of Meshtastic-compatible devices (CP210x, CH9102, CH340, ESP32-S3, nRF52840, etc.)
- **Channel management** — view, create, and delete mesh channels with configurable encryption (AES128/AES256/simple/none)
- **Real-time messaging** — send and receive text messages on any channel with broadcast or direct node targeting
- **Radio metrics** — view SNR, RSSI, and hop count for each received message
- **Mesh node discovery** — see all nodes in the mesh with their name, hardware model, battery level, and GPS position
- **Message persistence** — messages are saved locally and restored across app restarts
- **Connection status** — detailed step-by-step feedback during connection and configuration loading
- **Adaptive layout** — responsive split-view UI that works on both desktop and mobile form factors

## Screenshots

_Coming soon_

## Building

### Prerequisites

- Rust (stable toolchain)
- meson >= 0.59
- GTK >= 4.12
- libadwaita >= 1.4
- blueprint-compiler

On Fedora:

```shell
sudo dnf install meson gtk4-devel libadwaita-devel blueprint-compiler
```

On Arch Linux:

```shell
sudo pacman -S meson gtk4 libadwaita blueprint-compiler
```

### Build & Install

```shell
meson setup _build -Dprofile=development
ninja -C _build
sudo ninja -C _build install
```

### Run

```shell
paper-mesh
```

Make sure your user is in the `dialout` group for serial port access:

```shell
sudo usermod -aG dialout $USER
# Log out and back in for the change to take effect
```

## How It Works

Paper Mesh communicates with a Meshtastic radio using the [meshtastic](https://crates.io/crates/meshtastic) Rust crate over the device's serial (USB) or TCP interface. The radio handles all LoRa mesh networking — Paper Mesh is the user-facing chat client.

### Architecture

```
GTK4/libadwaita UI  <-->  GLib main loop  <--(async-channel)-->  Tokio runtime  <-->  Meshtastic radio
```

- **Model layer** — GObject-based models for Device, Node, Channel, and Message that integrate with GTK's reactive property/signal system
- **Device bridge** — background Tokio thread handles the meshtastic protocol, communicates with the GTK main loop via async channels
- **UI layer** — Blueprint-defined templates with Rust widget implementations following GNOME HIG

### Message Persistence

Messages are stored as JSON files per channel in `~/.local/share/paper-mesh/messages/`. Up to 1000 messages per channel are retained.

## License

GPL-3.0-or-later
