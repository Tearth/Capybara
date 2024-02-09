#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]

use capybara::anyhow::Result;
use capybara::app::ApplicationContext;
use capybara::app::ApplicationState;
use capybara::assets::loader::AssetsLoader;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui::panel::Side;
use capybara::egui::Color32;
use capybara::egui::FontFamily;
use capybara::egui::FontId;
use capybara::egui::FullOutput;
use capybara::egui::Id;
use capybara::egui::RawInput;
use capybara::egui::RichText;
use capybara::egui::SidePanel;
use capybara::egui::Slider;
use capybara::egui::TextStyle;
use capybara::fast_gpu;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use capybara::network::packet::Packet;
use capybara::renderer::sprite::Sprite;
use capybara::renderer::sprite::TextureId;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::Coordinates;
use capybara::window::InputEvent;
use capybara::window::Key;
use capybara::window::WindowStyle;
use simple_base::PacketSetViewport;
use simple_base::*;
use std::collections::VecDeque;

fast_gpu!();

#[derive(Default)]
struct GlobalData {
    assets: AssetsLoader,
}

#[derive(Default)]
struct MainScene {
    objects: Vec<Object>,
    objects_count: u32,
    objects_last_update: Option<Instant>,
    ping_last_update: Option<Instant>,
    delta_history: VecDeque<f32>,
    tick_history: VecDeque<f32>,
    initialized: bool,

    client: WebSocketClient,
}

struct Object {
    previous_position: Vec2,
    current_position: Vec2,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::WindowSizeChange { size } = event {
            if *self.client.status.read().unwrap() == ConnectionStatus::Connected {
                self.client.send_packet(Packet::from_object(PACKET_SET_VIEWPORT, &PacketSetViewport { size: size.into() }));
            }
        } else if let InputEvent::KeyPress { key: Key::Escape, repeat: _, modifiers: _ } = event {
            state.window.close();
        }

        Ok(())
    }

    fn fixed(&mut self, _state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _accumulator: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.delta_history.push_back(delta);

        if self.delta_history.len() > 100 {
            self.delta_history.pop_front();
        }

        if !self.initialized && state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None);
            state.ui.instantiate_assets(&state.global.assets, None);

            self.client.connect("ws://localhost:9999");
            self.initialized = true;
        }

        let now = Instant::now();

        if *self.client.status.read().unwrap() == ConnectionStatus::Connected {
            if self.client.has_connected() {
                let size = state.renderer.viewport_size;
                self.client.send_packet(Packet::from_object(PACKET_SET_VIEWPORT, &PacketSetViewport { size }));
            }

            while let Some(packet) = self.client.poll_packet() {
                match packet.get_id() {
                    Some(PACKET_OBJECTS_ARRAY) => {
                        let positions = packet.to_array::<Vec2>().unwrap();

                        // Same size, so we can properly update current and previous positions
                        if positions.len() == self.objects.len() {
                            for i in 0..self.objects.len() {
                                self.objects[i].previous_position = self.objects[i].current_position;
                                self.objects[i].current_position = positions[i];
                            }
                        // Different size, reset all objects
                        } else {
                            self.objects = positions.iter().map(|p| Object { previous_position: *p, current_position: *p }).collect();
                        }

                        self.tick_history.push_back((now - self.objects_last_update.unwrap_or(now)).as_millis() as f32);

                        if self.tick_history.len() > 20 {
                            self.tick_history.pop_front();
                        }

                        self.objects_last_update = Some(now);
                    }
                    Some(PACKET_SET_COUNT) => {
                        self.objects_count = packet.to_object::<PacketSetCount>().unwrap().count;
                    }
                    _ => {}
                }
            }

            if self.ping_last_update.is_none() {
                self.ping_last_update = Some(now);
            }

            if (now - self.ping_last_update.unwrap_or(now)).as_millis() > 250 {
                self.client.send_ping();
                self.ping_last_update = Some(now);
            }
        }

        if self.initialized {
            let texture_id = state.renderer.textures.get_id("Takodachi")?;
            let alpha = (now - self.objects_last_update.unwrap_or(now)).as_millis() as f32 / TICK as f32;

            for object in &mut self.objects {
                state.renderer.draw_sprite(&Sprite {
                    position: object.current_position * alpha + object.previous_position * (1.0 - alpha),
                    texture_id: TextureId::Some(texture_id),
                    ..Default::default()
                });
            }
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side")).exact_width(160.0).resizable(false).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Monospace };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);
                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let label = format!("Ping: {} ms", *self.client.ping.read().unwrap());
                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let tick_average = self.tick_history.iter().sum::<f32>() / self.tick_history.len() as f32;
                    let label = format!("Tick: {:.2} ms", tick_average);
                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    ui.style_mut().drag_value_text_style = TextStyle::Monospace;
                    ui.style_mut().text_styles.get_mut(&TextStyle::Monospace).unwrap().size = 20.0;

                    ui.add_space(10.0);
                    ui.label(RichText::new("Objects count:").font(font.clone()).heading().color(color));
                    if ui.add(Slider::new(&mut self.objects_count, 0..=10000).text_color(color).logarithmic(true)).changed() {
                        if *self.client.status.read().unwrap() == ConnectionStatus::Connected {
                            self.client.send_packet(Packet::from_object(PACKET_SET_COUNT, &PacketSetCount { count: self.objects_count }));
                        }
                    }
                }
            });
        });

        Ok((output, None))
    }

    fn reset(&self) -> Box<dyn Scene<GlobalData>> {
        Box::<Self>::default()
    }
}

fn main() {
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("Network benchmark", WindowStyle::Window { size: Coordinates::new(1280, 720) }, Some(4))?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
