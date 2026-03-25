//! Persistent message storage using JSON files.
//!
//! Messages are stored per-channel in:
//!   ~/.local/share/paper-mesh/messages/channel_{index}.json

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{MeshMessage, MessageDirection};
use crate::utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StoredMessage {
    pub packet_id: u32,
    pub from_node: u32,
    pub to_node: u32,
    pub channel_index: u32,
    pub text: String,
    pub timestamp: u32,
    pub direction: String, // "incoming" or "outgoing"
    pub snr: f32,
    pub rssi: i32,
    pub hop_start: u32,
    pub hop_limit: u32,
    pub sender_name: String,
}

impl StoredMessage {
    pub(crate) fn from_mesh_message(msg: &MeshMessage) -> Self {
        Self {
            packet_id: msg.packet_id(),
            from_node: msg.from_node(),
            to_node: msg.to_node(),
            channel_index: msg.channel_index(),
            text: msg.text(),
            timestamp: msg.timestamp(),
            direction: match msg.direction() {
                MessageDirection::Incoming => "incoming".to_string(),
                MessageDirection::Outgoing => "outgoing".to_string(),
            },
            snr: msg.snr(),
            rssi: msg.rssi(),
            hop_start: 0,
            hop_limit: 0,
            sender_name: msg.sender_name(),
        }
    }

    pub(crate) fn to_mesh_message(&self) -> MeshMessage {
        let direction = match self.direction.as_str() {
            "outgoing" => MessageDirection::Outgoing,
            _ => MessageDirection::Incoming,
        };
        let msg = MeshMessage::new(
            self.packet_id,
            self.from_node,
            self.to_node,
            self.channel_index,
            &self.text,
            self.timestamp,
            direction,
        );
        msg.set_radio_info(self.snr, self.rssi, self.hop_start, self.hop_limit);
        msg.set_sender_name(&self.sender_name);
        msg
    }
}

fn messages_dir() -> PathBuf {
    let dir = utils::data_dir().join("messages");
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log::error!("Failed to create messages dir: {e}");
        }
    }
    dir
}

fn channel_file(channel_index: u32) -> PathBuf {
    messages_dir().join(format!("channel_{}.json", channel_index))
}

/// Load stored messages for a channel
pub(crate) fn load_messages(channel_index: u32) -> Vec<StoredMessage> {
    let path = channel_file(channel_index);
    if !path.exists() {
        return vec![];
    }
    match fs::read_to_string(&path) {
        Ok(data) => match serde_json::from_str::<Vec<StoredMessage>>(&data) {
            Ok(messages) => {
                log::info!(
                    "Loaded {} messages for channel {}",
                    messages.len(),
                    channel_index
                );
                messages
            }
            Err(e) => {
                log::warn!("Failed to parse messages for channel {}: {e}", channel_index);
                vec![]
            }
        },
        Err(e) => {
            log::warn!("Failed to read messages for channel {}: {e}", channel_index);
            vec![]
        }
    }
}

/// Append a single message to the channel's store
pub(crate) fn append_message(channel_index: u32, msg: &MeshMessage) {
    let path = channel_file(channel_index);
    let mut messages = load_messages(channel_index);

    // Deduplicate by packet_id (skip if already stored)
    let stored = StoredMessage::from_mesh_message(msg);
    if messages
        .iter()
        .any(|m| m.packet_id == stored.packet_id && m.packet_id != 0)
    {
        return;
    }

    messages.push(stored);

    // Keep only last 1000 messages per channel
    if messages.len() > 1000 {
        messages.drain(..messages.len() - 1000);
    }

    if let Err(e) = write_messages(channel_index, &messages) {
        log::error!("Failed to save messages for channel {}: {e}", channel_index);
    }
}

fn write_messages(channel_index: u32, messages: &[StoredMessage]) -> anyhow::Result<()> {
    let path = channel_file(channel_index);
    let data = serde_json::to_string(messages)?;
    fs::write(path, data)?;
    Ok(())
}
