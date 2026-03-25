//! Persistent message storage using JSON files.
//!
//! Messages are stored per-channel in:
//!   <data_dir>/messages/channel_{index}.json

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{DeliveryStatus, MeshMessage, MessageDirection};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// "none", "sending", "delivered", "failed"
    #[serde(default)]
    pub delivery_status: String,
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
            delivery_status: match msg.delivery_status() {
                DeliveryStatus::None => "none".to_string(),
                DeliveryStatus::Sending => "sending".to_string(),
                DeliveryStatus::Delivered => "delivered".to_string(),
                DeliveryStatus::Failed => "failed".to_string(),
            },
        }
    }

    pub(crate) fn to_mesh_message(&self) -> MeshMessage {
        let direction = match self.direction.as_str() {
            "outgoing" => MessageDirection::Outgoing,
            _ => MessageDirection::Incoming,
        };
        let delivery = match self.delivery_status.as_str() {
            "sending" => DeliveryStatus::Failed, // stale "sending" from previous session = failed
            "delivered" => DeliveryStatus::Delivered,
            "failed" => DeliveryStatus::Failed,
            _ => DeliveryStatus::None,
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
        msg.set_delivery_status(delivery);
        msg
    }
}

fn messages_dir() -> PathBuf {
    let dir = crate::utils::data_dir().join("messages");
    ensure_dir(&dir);
    dir
}

fn ensure_dir(dir: &PathBuf) {
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(dir) {
            log::error!("Failed to create messages dir: {e}");
        }
    }
}

fn channel_file(channel_index: u32) -> PathBuf {
    messages_dir().join(format!("channel_{}.json", channel_index))
}

fn channel_file_in(dir: &PathBuf, channel_index: u32) -> PathBuf {
    dir.join(format!("channel_{}.json", channel_index))
}

/// Load stored messages for a channel
pub(crate) fn load_messages(channel_index: u32) -> Vec<StoredMessage> {
    load_messages_from(&channel_file(channel_index))
}

/// Load stored messages from a specific path
pub(crate) fn load_messages_from(path: &PathBuf) -> Vec<StoredMessage> {
    if !path.exists() {
        return vec![];
    }
    match fs::read_to_string(path) {
        Ok(data) => match serde_json::from_str::<Vec<StoredMessage>>(&data) {
            Ok(messages) => messages,
            Err(e) => {
                log::warn!("Failed to parse messages: {e}");
                vec![]
            }
        },
        Err(e) => {
            log::warn!("Failed to read messages: {e}");
            vec![]
        }
    }
}

/// Append a single message to the channel's store
pub(crate) fn append_message(channel_index: u32, msg: &MeshMessage) {
    append_message_to(&channel_file(channel_index), msg);
}

/// Append a single message to a specific file path
pub(crate) fn append_message_to(path: &PathBuf, msg: &MeshMessage) {
    let mut messages = load_messages_from(path);

    let stored = StoredMessage::from_mesh_message(msg);
    // Deduplicate by packet_id (skip if already stored)
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

    if let Err(e) = write_messages_to(path, &messages) {
        log::error!("Failed to save messages: {e}");
    }
}

/// Update the delivery status of a persisted message by packet_id
pub(crate) fn update_delivery_status(channel_index: u32, packet_id: u32, status: &str) {
    let path = channel_file(channel_index);
    let mut messages = load_messages_from(&path);
    let mut changed = false;
    for m in &mut messages {
        if m.packet_id == packet_id && packet_id != 0 {
            m.delivery_status = status.to_string();
            changed = true;
            break;
        }
    }
    if changed {
        if let Err(e) = write_messages_to(&path, &messages) {
            log::error!("Failed to update delivery status: {e}");
        }
    }
}

fn write_messages_to(path: &PathBuf, messages: &[StoredMessage]) -> anyhow::Result<()> {
    let data = serde_json::to_string(messages)?;
    fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_gtk() { crate::test_helpers::init_gtk(); }

    fn make_message(packet_id: u32, text: &str, channel: u32, direction: MessageDirection) -> MeshMessage {
        let msg = MeshMessage::new(packet_id, 100, 200, channel, text, 1000, direction);
        msg.set_sender_name("TestNode");
        msg.set_radio_info(-5.0, -100, 3, 1);
        msg
    }

    #[test]
    fn test_stored_message_roundtrip() {
        init_gtk();
        let msg = make_message(42, "hello mesh", 0, MessageDirection::Incoming);
        let stored = StoredMessage::from_mesh_message(&msg);

        assert_eq!(stored.packet_id, 42);
        assert_eq!(stored.text, "hello mesh");
        assert_eq!(stored.from_node, 100);
        assert_eq!(stored.to_node, 200);
        assert_eq!(stored.channel_index, 0);
        assert_eq!(stored.timestamp, 1000);
        assert_eq!(stored.direction, "incoming");
        assert_eq!(stored.sender_name, "TestNode");
        assert_eq!(stored.snr, -5.0);
        assert_eq!(stored.rssi, -100);

        let restored = stored.to_mesh_message();
        assert_eq!(restored.packet_id(), 42);
        assert_eq!(restored.text(), "hello mesh");
        assert_eq!(restored.from_node(), 100);
        assert_eq!(restored.to_node(), 200);
        assert_eq!(restored.direction(), MessageDirection::Incoming);
        assert_eq!(restored.sender_name(), "TestNode");
    }

    #[test]
    fn test_stored_message_outgoing_direction() {
        init_gtk();
        let msg = make_message(1, "out", 0, MessageDirection::Outgoing);
        let stored = StoredMessage::from_mesh_message(&msg);
        assert_eq!(stored.direction, "outgoing");

        let restored = stored.to_mesh_message();
        assert_eq!(restored.direction(), MessageDirection::Outgoing);
    }

    #[test]
    fn test_stored_message_unknown_direction_defaults_incoming() {
        init_gtk();
        let stored = StoredMessage {
            packet_id: 1,
            from_node: 0,
            to_node: 0,
            channel_index: 0,
            text: "x".into(),
            timestamp: 0,
            direction: "garbage".into(),
            snr: 0.0,
            rssi: 0,
            hop_start: 0,
            hop_limit: 0,
            sender_name: "".into(),
            delivery_status: "none".into(),
        };
        let msg = stored.to_mesh_message();
        assert_eq!(msg.direction(), MessageDirection::Incoming);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let stored = StoredMessage {
            packet_id: 99,
            from_node: 1,
            to_node: 2,
            channel_index: 3,
            text: "test message".into(),
            timestamp: 12345,
            direction: "incoming".into(),
            snr: -3.5,
            rssi: -80,
            hop_start: 3,
            hop_limit: 1,
            sender_name: "Alice".into(),
            delivery_status: "none".into(),
        };
        let json = serde_json::to_string(&stored).unwrap();
        let deserialized: StoredMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(stored, deserialized);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let path = PathBuf::from("/tmp/nonexistent_paper_mesh_test_file.json");
        let msgs = load_messages_from(&path);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_load_from_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.json");
        fs::write(&path, "not valid json").unwrap();
        let msgs = load_messages_from(&path.to_path_buf());
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_append_and_load() {
        init_gtk();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("channel_0.json");

        let msg1 = make_message(1, "first", 0, MessageDirection::Incoming);
        let msg2 = make_message(2, "second", 0, MessageDirection::Outgoing);

        append_message_to(&path, &msg1);
        append_message_to(&path, &msg2);

        let loaded = load_messages_from(&path);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "first");
        assert_eq!(loaded[1].text, "second");
    }

    #[test]
    fn test_deduplication_by_packet_id() {
        init_gtk();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("channel_0.json");

        let msg = make_message(42, "hello", 0, MessageDirection::Incoming);
        append_message_to(&path, &msg);
        append_message_to(&path, &msg); // duplicate

        let loaded = load_messages_from(&path);
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn test_no_dedup_for_packet_id_zero() {
        init_gtk();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("channel_0.json");

        let msg1 = make_message(0, "a", 0, MessageDirection::Incoming);
        let msg2 = make_message(0, "b", 0, MessageDirection::Incoming);
        append_message_to(&path, &msg1);
        append_message_to(&path, &msg2);

        let loaded = load_messages_from(&path);
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_max_1000_messages() {
        init_gtk();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("channel_0.json");

        for i in 0..1005 {
            let msg = make_message(i + 1, &format!("msg {i}"), 0, MessageDirection::Incoming);
            append_message_to(&path, &msg);
        }

        let loaded = load_messages_from(&path);
        assert_eq!(loaded.len(), 1000);
        // Oldest messages should be trimmed
        assert_eq!(loaded[0].text, "msg 5");
        assert_eq!(loaded[999].text, "msg 1004");
    }
}
