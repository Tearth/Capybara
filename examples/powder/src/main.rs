#![allow(clippy::collapsible_if)]

use crate::core::selector::Selector;
use crate::core::*;
use capybara::anyhow::Result;
use capybara::app::ApplicationContext;
use capybara::app::ApplicationState;
use capybara::assets::loader::AssetsLoader;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui;
use capybara::egui::Button;
use capybara::egui::Color32;
use capybara::egui::FontFamily;
use capybara::egui::FontId;
use capybara::egui::Frame;
use capybara::egui::FullOutput;
use capybara::egui::Label;
use capybara::egui::RawInput;
use capybara::egui::RichText;
use capybara::egui::SidePanel;
use capybara::egui::Stroke;
use capybara::egui::TextStyle;
use capybara::fast_gpu;
use capybara::glam::IVec2;
use capybara::glam::Vec4;
use capybara::parking_lot::RwLock;
use capybara::powder::chunk::ParticleState;
use capybara::powder::simulation::PowderSimulation;
use capybara::powder::ParticleDefinition;
use capybara::rustc_hash::FxHashMap;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::ui::debug::DebugWindow;
use capybara::ui::debug::ProfilerPlotDefinition;
use capybara::utils::debug::DebugCollector;
use capybara::utils::debug::DebugConsole;
use capybara::utils::profiler::Profiler;
use capybara::window::InputEvent;
use capybara::window::Key;
use capybara::window::MouseButton;
use capybara::window::MouseWheelDirection;
use capybara::window::WindowStyle;
use std::sync::Arc;

pub mod core;

fast_gpu!();

#[derive(Default)]
struct GlobalData {
    assets: AssetsLoader,
}

struct MainScene {
    pub simulation: PowderSimulation,
    pub selector: Selector,

    pub rigidbody_mode: bool,
    pub initialized: bool,
    pub force_all_chunks: bool,

    pub debug_enabled: bool,
    pub debug_window: DebugWindow,
    pub debug_console: DebugConsole,
    pub debug_collector: DebugCollector,
    pub debug_profiler: Profiler,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        let definitions = vec![
            ParticleDefinition {
                name: "Sand".to_string(),
                state: ParticleState::Powder,
                color: Vec4::new(1.0, 1.0, 0.5, 1.0),
                density: 3.0,
                ..Default::default()
            },
            ParticleDefinition {
                name: "Stone".to_string(),
                state: ParticleState::Solid,
                color: Vec4::new(0.3, 0.3, 0.3, 1.0),
                mass: 4.0,
                ..Default::default()
            },
            ParticleDefinition {
                name: "Water".to_string(),
                state: ParticleState::Fluid,
                color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                mass: 0.0,
                density: 1.0,
                displacement: 0.6,
                drag: 0.5,
                compressibility: 0.1,
                fluidity: 4,
                extensibility: 0.80,
                hpressure_gradient_length: 10.0,
                hpressure_gradient_end: Vec4::new(0.0, 0.0, 0.3, 1.0),
            },
            ParticleDefinition {
                name: "Wood".to_string(),
                state: ParticleState::Solid,
                color: Vec4::new(0.7, 0.4, 0.15, 1.0),
                mass: 0.5,
                ..Default::default()
            },
        ];

        self.simulation.definitions = Arc::new(RwLock::new(definitions));
        self.simulation.reset(state.renderer, state.physics);

        self.debug_window.plot_definitions = vec![
            ProfilerPlotDefinition { name: String::from("input"), label: String::from("Input average"), color: Color32::RED },
            ProfilerPlotDefinition { name: String::from("fixed"), label: String::from("Fixed average"), color: Color32::GREEN },
            ProfilerPlotDefinition { name: String::from("frame"), label: String::from("Frame average"), color: Color32::LIGHT_BLUE },
            ProfilerPlotDefinition { name: String::from("ui"), label: String::from("UI average"), color: Color32::YELLOW },
        ];

        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        self.debug_profiler.resume("input");

        match event {
            InputEvent::MouseMove { position, modifiers: _ } => {
                self.selector.set_cursor_position(
                    state
                        .renderer
                        .cameras
                        .get(state.renderer.active_camera_id)
                        .unwrap()
                        .from_window_to_world_coordinates(position.as_vec2())
                        .as_ivec2(),
                );
            }
            InputEvent::MouseWheelRotated { direction, modifiers: _ } => match direction {
                MouseWheelDirection::Up => self.selector.decrease_size(),
                MouseWheelDirection::Down => self.selector.increase_size(),
                _ => {}
            },
            InputEvent::MouseButtonPress { button, position: _, modifiers: _ } => {
                if button == MouseButton::Left {
                    if self.rigidbody_mode {
                        let mut last_position = None;
                        let mut points = FxHashMap::default();

                        while let Some(position) = self.selector.get_next_selected_particle(last_position) {
                            if let Some(chunk) = self.simulation.get_chunk(position) {
                                let chunk = chunk.read();
                                if let Some(particle) = chunk.get_particle(position) {
                                    if !particle.structure {
                                        let definition = &self.simulation.definitions.read()[particle.r#type];
                                        points.insert(particle.position, definition.mass);
                                    }
                                }
                            }
                            last_position = Some(position);
                        }

                        self.simulation.create_structure(state.physics, &mut points);
                    }
                }
            }
            InputEvent::KeyPress { key, repeat: _, modifiers } => {
                if key == Key::KeyD && modifiers.shift {
                    self.debug_enabled = !self.debug_enabled;
                    self.debug_profiler.enabled = !self.debug_profiler.enabled;
                    self.debug_collector.enabled = !self.debug_collector.enabled;
                }
            }
            _ => (),
        }

        self.debug_profiler.pause("input");
        Ok(())
    }

    fn fixed(&mut self, state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        self.debug_profiler.start("fixed");
        self.simulation.apply_forces(state.physics);
        self.simulation.update_structures(state.physics);
        self.debug_profiler.stop("fixed");

        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _accumulator: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.debug_profiler.start("frame");

        if !self.initialized && state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(&state.global.assets, None);

            let mut style = (*state.ui.inner.read().style()).clone();
            style.text_styles = [
                (TextStyle::Heading, (FontId { size: 32.0, family: FontFamily::Monospace })),
                (TextStyle::Body, (FontId { size: 20.0, family: FontFamily::Monospace })),
                (TextStyle::Button, (FontId { size: 24.0, family: FontFamily::Monospace })),
                (TextStyle::Name("Debug".into()), (FontId { size: 20.0, family: FontFamily::Monospace })),
            ]
            .into();
            style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(240, 240, 240));
            state.ui.inner.read().set_style(style);

            self.initialized = true;
        }

        if state.window.cursor_position.x < state.window.size.x - 100 {
            if state.window.mouse_state[MouseButton::Left as usize] {
                if !self.rigidbody_mode {
                    self.selector.fill_selection(&mut self.simulation);
                }
            }

            if state.window.mouse_state[MouseButton::Right as usize] {
                self.selector.clear_selection(&mut self.simulation);
            }
        }

        self.simulation.logic(state.renderer, state.physics, self.force_all_chunks, delta);
        self.simulation.draw(state.renderer);
        self.selector.draw(state.renderer);

        if self.debug_enabled {
            self.simulation.draw_debug(state.renderer);
            state.physics.draw_debug(state.renderer, 50.0);

            self.debug_collector.collect(state.window, state.renderer, delta);
            self.process_console();
        }
        self.debug_profiler.stop("frame");

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        self.debug_profiler.start("ui");
        self.debug_profiler.stop("input");

        let output =
            state.ui.inner.read().run(input, |context| {
                if self.debug_enabled {
                    self.debug_window.show(context, &mut self.debug_console, &self.debug_profiler, &mut self.debug_collector);
                }

                SidePanel::left("panel-left").frame(Frame::none()).resizable(false).show_separator_line(false).exact_width(200.0).show(
                    context,
                    |ui| {
                        let selector_position = self.selector.position;

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.add_space(10.0);
                                ui.add(Label::new(RichText::new(format!("position: {:.2} {:.2}", selector_position.x, selector_position.y))));

                                if self.simulation.is_position_valid(selector_position) {
                                    if let Some(chunk) = self.simulation.get_chunk(selector_position) {
                                        let chunk = chunk.read();
                                        let particle = chunk.get_particle(selector_position);
                                        if let Some(particle) = particle {
                                            ui.add(Label::new(format!("velocity: {:.2} {:.2}", particle.velocity.x, particle.velocity.y)));
                                            ui.add(Label::new(format!("offset: {:.2} {:.2}", particle.offset.x, particle.offset.y)));
                                            ui.add(Label::new(format!("hpressure: {:.2}", particle.hpressure)));
                                        }
                                    }
                                }
                            });
                        })
                    },
                );

                SidePanel::right("panel-right").frame(Frame::none()).resizable(false).show_separator_line(false).show(context, |ui| {
                    let size = egui::Vec2::new(100.0, 0.0);
                    let definitions = self.simulation.definitions.read();

                    for (r#type, definition) in definitions.iter().enumerate() {
                        let selected = self.selector.particle_type == r#type;
                        if ui.add(Button::new(&definition.name).min_size(size).selected(selected)).clicked() {
                            self.selector.particle_type = r#type;
                            self.selector.particle_definition = Some((*definition).clone());
                            self.selector.update();
                            self.rigidbody_mode = false;
                        }
                    }
                    drop(definitions);

                    ui.add_space(20.0);

                    if ui.add(Button::new("Rigodbody").min_size(size).selected(self.rigidbody_mode)).clicked() {
                        self.selector.particle_type = usize::MAX;
                        self.selector.particle_definition = None;
                        self.selector.update();
                        self.rigidbody_mode = !self.rigidbody_mode;
                    }

                    ui.add_space(20.0);

                    if ui.add(Button::new("Load").min_size(size)).clicked() {
                        persistence::load("saves/test.lvl", &mut self.simulation, state.renderer, state.physics);
                    }

                    if ui.add(Button::new("Save").min_size(size)).clicked() {
                        persistence::save("saves/test.lvl", &mut self.simulation);
                    }

                    ui.add_space(20.0);

                    ui.checkbox(&mut self.force_all_chunks, "Force all");
                });
            });

        self.debug_profiler.stop("ui");
        Ok((output, None))
    }

    fn reset(&self) -> Box<dyn Scene<GlobalData>> {
        Box::<Self>::default()
    }
}

impl MainScene {
    pub fn process_console(&mut self) {
        while let Some(command) = self.debug_console.poll_command() {
            match command.to_lowercase().as_str() {
                "test" => self.debug_console.apply_output("Test"),
                _ => self.debug_console.apply_output("Invalid command"),
            }
        }
    }
}

impl Default for MainScene {
    fn default() -> Self {
        Self {
            simulation: PowderSimulation::new(CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER),
            selector: Default::default(),
            rigidbody_mode: Default::default(),
            initialized: Default::default(),
            force_all_chunks: Default::default(),
            debug_enabled: Default::default(),
            debug_window: Default::default(),
            debug_console: Default::default(),
            debug_collector: Default::default(),
            debug_profiler: Default::default(),
        }
    }
}

fn main() {
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("Powder", WindowStyle::Window { size: IVec2::new(1280, 720) }, Some(4))?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
