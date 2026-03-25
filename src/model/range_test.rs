use std::cell::Cell;
use std::cell::RefCell;
use std::time::Instant;

/// A single range test result
#[derive(Debug, Clone)]
pub(crate) struct RangeTestResult {
    pub sequence: u32,
    pub sent_at: Instant,
    pub rtt_ms: Option<u64>,
    pub snr: f32,
    pub rssi: i32,
    pub hops: u32,
}

/// Tracks a range test session
#[derive(Debug)]
pub(crate) struct RangeTestSession {
    target_node: Cell<u32>,
    total_sent: Cell<u32>,
    total_acked: Cell<u32>,
    results: RefCell<Vec<RangeTestResult>>,
    active: Cell<bool>,
}

impl Default for RangeTestSession {
    fn default() -> Self {
        Self {
            target_node: Cell::new(0),
            total_sent: Cell::new(0),
            total_acked: Cell::new(0),
            results: RefCell::new(Vec::new()),
            active: Cell::new(false),
        }
    }
}

impl RangeTestSession {
    pub(crate) fn new(target_node: u32) -> Self {
        let s = Self::default();
        s.target_node.set(target_node);
        s.active.set(true);
        s
    }

    pub(crate) fn target_node(&self) -> u32 {
        self.target_node.get()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.get()
    }

    pub(crate) fn stop(&self) {
        self.active.set(false);
    }

    pub(crate) fn record_sent(&self) -> u32 {
        let seq = self.total_sent.get() + 1;
        self.total_sent.set(seq);
        self.results.borrow_mut().push(RangeTestResult {
            sequence: seq,
            sent_at: Instant::now(),
            rtt_ms: None,
            snr: 0.0,
            rssi: 0,
            hops: 0,
        });
        seq
    }

    pub(crate) fn record_ack(&self, sequence: u32, snr: f32, rssi: i32, hops: u32) {
        self.total_acked.set(self.total_acked.get() + 1);
        let mut results = self.results.borrow_mut();
        if let Some(r) = results.iter_mut().find(|r| r.sequence == sequence) {
            r.rtt_ms = Some(r.sent_at.elapsed().as_millis() as u64);
            r.snr = snr;
            r.rssi = rssi;
            r.hops = hops;
        }
    }

    pub(crate) fn total_sent(&self) -> u32 {
        self.total_sent.get()
    }

    pub(crate) fn total_acked(&self) -> u32 {
        self.total_acked.get()
    }

    pub(crate) fn packet_loss_percent(&self) -> f32 {
        let sent = self.total_sent.get();
        if sent == 0 {
            return 0.0;
        }
        let lost = sent - self.total_acked.get();
        (lost as f32 / sent as f32) * 100.0
    }

    pub(crate) fn avg_rtt_ms(&self) -> Option<u64> {
        let results = self.results.borrow();
        let rtts: Vec<u64> = results.iter().filter_map(|r| r.rtt_ms).collect();
        if rtts.is_empty() {
            return None;
        }
        Some(rtts.iter().sum::<u64>() / rtts.len() as u64)
    }

    pub(crate) fn results(&self) -> Vec<RangeTestResult> {
        self.results.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let s = RangeTestSession::new(42);
        assert_eq!(s.target_node(), 42);
        assert!(s.is_active());
        assert_eq!(s.total_sent(), 0);
        assert_eq!(s.total_acked(), 0);
    }

    #[test]
    fn test_send_and_ack() {
        let s = RangeTestSession::new(1);
        let seq = s.record_sent();
        assert_eq!(seq, 1);
        assert_eq!(s.total_sent(), 1);

        s.record_ack(1, -5.0, -80, 2);
        assert_eq!(s.total_acked(), 1);
        assert!(s.avg_rtt_ms().is_some());
    }

    #[test]
    fn test_packet_loss() {
        let s = RangeTestSession::new(1);
        s.record_sent();
        s.record_sent();
        s.record_sent();
        s.record_ack(1, 0.0, 0, 0);
        // 3 sent, 1 acked = 66.7% loss
        assert!((s.packet_loss_percent() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_stop() {
        let s = RangeTestSession::new(1);
        assert!(s.is_active());
        s.stop();
        assert!(!s.is_active());
    }
}
