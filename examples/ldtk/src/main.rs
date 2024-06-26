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
use capybara::fast_gpu;
use capybara::glam::IVec2;
use capybara::glam::Vec2;
use capybara::renderer::sprite::Sprite;
use capybara::renderer::sprite::TextureId;
use capybara::renderer::sprite::TextureType;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::InputEvent;
use capybara::window::Key;
use capybara::window::WindowStyle;
use std::collections::VecDeque;

fast_gpu!();

#[derive(Default)]
struct GlobalData {
    assets: AssetsLoader,
}

#[derive(Default)]
struct MainScene {
    initialized: bool,
    delta_history: VecDeque<f32>,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key: Key::Escape, repeat: _, modifiers: _ } = event {
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
            state.window.set_swap_interval(0);

            self.initialized = true;
        }

        if self.initialized {
            let world = &state.global.assets.worlds[0];
            let level = &world.levels[0];

            for layer in &level.layers {
                if let Some(tilemap_id) = layer.tilemap_id {
                    let tilemap = world.tilemaps.iter().find(|p| p.id == tilemap_id).unwrap();
                    for tile in &layer.tiles {
                        state.renderer.draw_sprite(&Sprite {
                            position: tile.position + Vec2::new(140.0, 0.0),
                            size: Some(tilemap.tile_size),
                            anchor: Vec2::ZERO,
                            texture_id: TextureId::Some(state.renderer.textures.get_id(&tilemap.name).unwrap()),
                            texture_type: TextureType::SimpleCoordinates { position: tile.source, size: tilemap.tile_size },
                            ..Default::default()
                        });
                    }
                }

                for entity in &layer.entities {
                    let tilemap = world.tilemaps.iter().find(|p| p.id == entity.tilemap_id).unwrap();
                    state.renderer.draw_sprite(&Sprite {
                        position: entity.position + Vec2::new(140.0, 0.0),
                        size: Some(tilemap.tile_size),
                        anchor: Vec2::ZERO,
                        texture_id: TextureId::Some(state.renderer.textures.get_id(&tilemap.name).unwrap()),
                        texture_type: TextureType::SimpleCoordinates { position: entity.source, size: tilemap.tile_size },
                        ..Default::default()
                    });
                }
            }
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side_panel")).exact_width(120.0).resizable(false).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Monospace };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));
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
    ApplicationContext::<GlobalData>::new("LDtk", WindowStyle::Window { size: IVec2::new(1280, 720) }, Some(4))?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
