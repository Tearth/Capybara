use crate::scenes::GlobalData;
use capybara::app::ApplicationState;
use capybara::instant::Instant;
use std::collections::VecDeque;

pub struct DebugCollector {
    pub fps_average: u32,
    pub delta_history: VecDeque<f32>,
    pub delta_history_capacity: usize,
    pub hardware_info: Option<String>,

    pub cache: Option<DebugCollectorData>,
    pub cache_timestamp: Option<Instant>,
    pub cache_lifetime: f32,
}

#[derive(Clone)]
pub struct DebugCollectorData {
    pub fps_current: f32,
    pub fps_average: u32,
    pub delta_current: f32,
    pub delta_average: f32,
    pub delta_deviation: f32,
    pub hardware_info: String,
}

impl DebugCollector {
    pub fn new() -> Self {
        Self {
            fps_average: 0,
            delta_history: VecDeque::new(),
            delta_history_capacity: 400,
            hardware_info: None,

            cache: None,
            cache_timestamp: None,
            cache_lifetime: 0.1,
        }
    }

    pub fn collect(&mut self, state: &ApplicationState<GlobalData>, delta: f32) {
        self.fps_average = state.renderer.fps;
        self.delta_history.push_back(delta);

        if self.delta_history.len() > self.delta_history_capacity {
            self.delta_history.pop_front();
        }

        if self.hardware_info.is_none() {
            self.hardware_info = Some(state.renderer.get_hardware_info());
        }
    }

    pub fn get_data(&mut self) -> DebugCollectorData {
        let now = Instant::now();

        if let Some(cache_timestamp) = self.cache_timestamp {
            if (now - cache_timestamp).as_secs_f32() < self.cache_lifetime {
                if let Some(cache) = &self.cache {
                    return cache.clone();
                }
            }
        }

        let delta = *self.delta_history.back().unwrap_or(&0.0);

        let fps_current = 1.0 / delta;
        let fps_average = self.fps_average;
        let delta_current = delta;
        let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
        let delta_deviation = (self.delta_history.iter().fold(0.0, |acc, p| acc + (p - delta).powi(2)) / self.delta_history.len() as f32).sqrt();
        let hardware_info = self.hardware_info.as_ref().cloned().unwrap_or("".to_string());

        self.cache = Some(DebugCollectorData { fps_current, fps_average, delta_current, delta_average, delta_deviation, hardware_info });
        self.cache_timestamp = Some(now);

        return self.cache.as_ref().cloned().unwrap();
    }
}

impl Default for DebugCollector {
    fn default() -> Self {
        Self::new()
    }
}
