use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::telemetry::Telemetry;

pub struct UAVState {
    pub current: Telemetry,
    pub previous: Option<Telemetry>,
    pub last_seen: Instant,
}

pub struct TelemetryBuffer {
    uavs: HashMap<String, UAVState>,
}

impl TelemetryBuffer {
    pub fn new() -> Self {
        Self {
            uavs: HashMap::new(),
        }
    }

    pub fn update(&mut self, telemetry: Telemetry) {
        let sn = telemetry.sn.clone();

        self.uavs
            .entry(sn)
            .and_modify(|state| {
                state.previous = Some(std::mem::replace(&mut state.current, telemetry.clone()));
                state.last_seen = Instant::now();
            })
            .or_insert(UAVState {
                current: telemetry,
                previous: None,
                last_seen: Instant::now(),
            });
    }

    pub fn cleanup_stale_uavs(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.uavs
            .retain(|_, state| now.duration_since(state.last_seen) < max_age);
    }

    pub fn get_all_uavs(&self) -> impl Iterator<Item = &UAVState> {
        self.uavs.values()
    }
}
