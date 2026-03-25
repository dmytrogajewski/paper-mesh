use std::cell::RefCell;

/// A single telemetry data point
#[derive(Debug, Clone)]
pub(crate) struct TelemetryPoint {
    pub timestamp: u32,
    pub battery_level: u32,
    pub voltage: f32,
    pub channel_utilization: f32,
    pub air_util_tx: f32,
}

/// Telemetry history for a single node
#[derive(Debug, Default)]
pub(crate) struct TelemetryHistory {
    points: RefCell<Vec<TelemetryPoint>>,
}

impl TelemetryHistory {
    pub(crate) fn add(&self, point: TelemetryPoint) {
        let mut points = self.points.borrow_mut();
        points.push(point);
        // Keep last 200 data points
        if points.len() > 200 {
            let excess = points.len() - 200;
            points.drain(..excess);
        }
    }

    pub(crate) fn points(&self) -> Vec<TelemetryPoint> {
        self.points.borrow().clone()
    }

    pub(crate) fn latest(&self) -> Option<TelemetryPoint> {
        self.points.borrow().last().cloned()
    }

    pub(crate) fn len(&self) -> usize {
        self.points.borrow().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_history() {
        let h = TelemetryHistory::default();
        assert_eq!(h.len(), 0);
        assert!(h.latest().is_none());
        assert!(h.points().is_empty());
    }

    #[test]
    fn test_add_and_retrieve() {
        let h = TelemetryHistory::default();
        h.add(TelemetryPoint {
            timestamp: 1000,
            battery_level: 85,
            voltage: 3.9,
            channel_utilization: 5.0,
            air_util_tx: 1.2,
        });
        assert_eq!(h.len(), 1);
        let latest = h.latest().unwrap();
        assert_eq!(latest.battery_level, 85);
        assert_eq!(latest.timestamp, 1000);
    }

    #[test]
    fn test_max_200_points() {
        let h = TelemetryHistory::default();
        for i in 0..250 {
            h.add(TelemetryPoint {
                timestamp: i,
                battery_level: 50,
                voltage: 3.7,
                channel_utilization: 0.0,
                air_util_tx: 0.0,
            });
        }
        assert_eq!(h.len(), 200);
        // Oldest should be trimmed
        assert_eq!(h.points()[0].timestamp, 50);
    }
}
