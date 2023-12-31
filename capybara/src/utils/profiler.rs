use instant::Instant;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

pub struct Profiler {
    pub enabled: bool,
    pub data: FxHashMap<String, ProfilerData>,
    pub history_capacity: usize,
}

#[derive(Default)]
pub struct ProfilerData {
    pub history: VecDeque<f32>,
    pub timestamp: Option<Instant>,
    pub accumulator: f32,
}

impl Profiler {
    pub fn new() -> Self {
        Self { enabled: false, data: Default::default(), history_capacity: 400 }
    }

    pub fn start(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(data) = self.data.get_mut(name) {
            data.timestamp = Some(Instant::now());
            data.accumulator = 0.0;
        } else {
            self.data
                .insert(name.to_string(), ProfilerData { history: Default::default(), timestamp: Some(Instant::now()), accumulator: 0.0 });
        }
    }

    pub fn stop(&mut self, name: &str) {
        if let Some(data) = self.data.get_mut(name) {
            if let Some(timestamp) = data.timestamp {
                data.history.push_back(data.accumulator + (Instant::now() - timestamp).as_secs_f32());
                data.timestamp = None;
            } else {
                if self.enabled {
                    data.history.push_back(data.accumulator);
                }
            }

            if data.history.len() > self.history_capacity {
                data.history.pop_front();
            }

            data.accumulator = 0.0;
        }
    }

    pub fn pause(&mut self, name: &str) {
        if let Some(data) = self.data.get_mut(name) {
            if let Some(timestamp) = data.timestamp {
                data.accumulator += (Instant::now() - timestamp).as_secs_f32();
                data.timestamp = None;
            }
        }
    }
    pub fn resume(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(data) = self.data.get_mut(name) {
            data.timestamp = Some(Instant::now());
        } else {
            self.data
                .insert(name.to_string(), ProfilerData { history: Default::default(), timestamp: Some(Instant::now()), accumulator: 0.0 });
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
