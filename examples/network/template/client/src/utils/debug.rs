use crate::scenes::GlobalData;
use capybara::app::ApplicationState;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::renderer::context::RendererStatistics;
use std::cmp;
use std::collections::VecDeque;

pub struct DebugCollector {
    pub enabled: bool,

    pub fps_average: u32,
    pub delta_history: VecDeque<f32>,
    pub delta_history_capacity: usize,
    pub private_memory_history: VecDeque<usize>,
    pub private_memory_peak: usize,
    pub reserved_memory_history: VecDeque<usize>,
    pub reserved_memory_peak: usize,
    pub memory_history_capacity: usize,
    pub hardware_info: Option<String>,
    pub resolution: Vec2,
    pub renderer: RendererStatistics,

    pub cache: Option<DebugCollectorData>,
    pub cache_timestamp: Option<Instant>,
    pub cache_lifetime: f32,
}

#[derive(Clone)]
pub struct DebugCollectorData {
    pub fps_average: u32,
    pub delta_current: f32,
    pub delta_average: f32,
    pub delta_deviation: f32,
    pub private_memory_current: usize,
    pub private_memory_peak: usize,
    pub reserved_memory_current: usize,
    pub reserved_memory_peak: usize,
    pub hardware_info: String,
    pub resolution: Vec2,
    pub renderer: RendererStatistics,
}

impl DebugCollector {
    pub fn new() -> Self {
        Self {
            enabled: false,

            fps_average: 0,
            delta_history: VecDeque::new(),
            delta_history_capacity: 400,
            private_memory_history: VecDeque::new(),
            reserved_memory_history: VecDeque::new(),
            private_memory_peak: 0,
            reserved_memory_peak: 0,
            memory_history_capacity: 400,
            hardware_info: None,
            resolution: Vec2::ZERO,
            renderer: Default::default(),

            cache: None,
            cache_timestamp: None,
            cache_lifetime: 0.1,
        }
    }

    pub fn collect(&mut self, state: &ApplicationState<GlobalData>, delta: f32) {
        if !self.enabled {
            return;
        }

        self.fps_average = state.renderer.fps;
        self.delta_history.push_back(delta);

        if self.delta_history.len() > self.delta_history_capacity {
            self.delta_history.pop_front();
        }

        let memory_usage = state.window.get_memory_usage();
        self.private_memory_history.push_back(memory_usage.private);
        self.reserved_memory_history.push_back(memory_usage.reserved);

        if self.private_memory_history.len() > self.memory_history_capacity {
            self.private_memory_history.pop_front();
        }

        if self.reserved_memory_history.len() > self.memory_history_capacity {
            self.reserved_memory_history.pop_front();
        }

        self.private_memory_peak = cmp::max(self.private_memory_peak, memory_usage.private);
        self.reserved_memory_peak = cmp::max(self.reserved_memory_peak, memory_usage.reserved);

        if self.hardware_info.is_none() {
            self.hardware_info = Some(state.renderer.get_hardware_info());
        }

        self.resolution = state.renderer.viewport_size;
        self.renderer = state.renderer.statistics;
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

        let fps_average = self.fps_average;
        let delta_current = delta;
        let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
        let delta_deviation = (self.delta_history.iter().fold(0.0, |acc, p| acc + (p - delta).powi(2)) / self.delta_history.len() as f32).sqrt();
        let private_memory_current = *self.private_memory_history.back().unwrap_or(&0);
        let private_memory_peak = self.private_memory_peak;
        let reserved_memory_current = *self.reserved_memory_history.back().unwrap_or(&0);
        let reserved_memory_peak = self.reserved_memory_peak;
        let hardware_info = self.hardware_info.as_ref().cloned().unwrap_or("".to_string());
        let resolution = self.resolution;
        let renderer = self.renderer;

        let data = DebugCollectorData {
            fps_average,
            delta_current,
            delta_average,
            delta_deviation,
            private_memory_current,
            private_memory_peak,
            reserved_memory_current,
            reserved_memory_peak,
            hardware_info,
            resolution,
            renderer,
        };
        self.cache = Some(data.clone());
        self.cache_timestamp = Some(now);

        data
    }
}

impl Default for DebugCollector {
    fn default() -> Self {
        Self::new()
    }
}
