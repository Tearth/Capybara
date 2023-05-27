use crate::assets::loader::AssetsLoader;
use crate::assets::RawTexture;
use crate::renderer::camera::Camera;
use crate::renderer::camera::CameraOrigin;
use crate::renderer::context::RendererContext;
use crate::renderer::sprite::Shape;
use crate::renderer::sprite::ShapeData;
use crate::renderer::sprite::Sprite;
use crate::renderer::texture::Texture;
use crate::renderer::texture::TextureFilter;
use crate::window::InputEvent;
use crate::window::Key;
use crate::window::Modifiers;
use crate::window::MouseButton;
use crate::window::MouseWheelDirection;
use anyhow::Result;
use core::slice;
use egui::epaint::ahash::HashMap;
use egui::epaint::Primitive;
use egui::Color32;
use egui::ColorImage;
use egui::Event;
use egui::FontData;
use egui::FontDefinitions;
use egui::FontFamily;
use egui::FullOutput;
use egui::ImageData;
use egui::PointerButton;
use egui::Pos2;
use egui::RawInput;
use egui::Rect;
use egui::TextureHandle;
use egui::TextureId;
use egui::TextureOptions;
use glam::Vec2;
use glow::HasContext;
use instant::Instant;

pub struct UiContext {
    pub inner: egui::Context,
    pub screen_size: Vec2,
    pub collected_events: Vec<Event>,
    pub modifiers: Modifiers,

    pub camera_id: usize,
    pub textures: HashMap<TextureId, usize>,
    pub handles: HashMap<String, TextureHandle>,

    time: Instant,
    max_texture_size: i32,
}

impl UiContext {
    pub fn new(renderer: &mut RendererContext) -> Result<Self> {
        Ok(Self {
            inner: Default::default(),
            screen_size: Default::default(),
            collected_events: Default::default(),
            modifiers: Default::default(),

            camera_id: renderer.cameras.store(Camera::new(Default::default(), renderer.viewport_size, CameraOrigin::LeftTop)),
            textures: Default::default(),
            handles: Default::default(),

            time: Instant::now(),
            max_texture_size: unsafe { renderer.gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
        })
    }

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader, prefix: Option<&str>) -> Result<()> {
        for texture in &assets.raw_textures {
            if let Some(prefix) = &prefix {
                if !texture.path.starts_with(prefix) {
                    continue;
                }
            }

            let size = [texture.size.x as usize, texture.size.y as usize];
            let mut image = ColorImage::new(size, Color32::TRANSPARENT);

            for x in 0..size[0] {
                for y in 0..size[1] {
                    let base = x * 4 + y * 4 * size[0];
                    let r = texture.data[base + 0];
                    let g = texture.data[base + 1];
                    let b = texture.data[base + 2];
                    let a = texture.data[base + 3];

                    image.pixels[x + y * size[0]] = Color32::from_rgba_premultiplied(r, g, b, a);
                }
            }

            let handle = self.inner.load_texture(texture.name.clone(), image, Default::default());
            self.handles.insert(texture.name.clone(), handle);
        }

        let mut fonts = FontDefinitions::default();

        for font in &assets.raw_fonts {
            if let Some(prefix) = &prefix {
                if !font.path.starts_with(prefix) {
                    continue;
                }
            }

            fonts.font_data.insert(font.name.clone(), FontData::from_owned(font.data.clone()));
            fonts.families.insert(FontFamily::Name(font.name.clone().into()), vec![font.name.clone()]);
        }

        self.inner.set_fonts(fonts);
        Ok(())
    }

    pub fn collect_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::WindowSizeChange { size } => {
                self.screen_size = Vec2::new(size.x as f32, size.y as f32);
            }
            InputEvent::MouseMove { position, modifiers } => {
                self.collected_events.push(Event::PointerMoved(Pos2::new(position.x as f32, position.y as f32)));
                self.modifiers = *modifiers;
            }
            InputEvent::MouseLeave => {
                self.collected_events.push(Event::PointerGone);
            }
            InputEvent::MouseButtonPress { button, position, modifiers } => {
                if let Some(egui_button) = map_mouse_button(*button) {
                    let egui_position = Pos2::new(position.x as f32, position.y as f32);
                    let egui_modifiers = map_modifiers(*modifiers);

                    self.collected_events.push(Event::PointerButton {
                        pos: egui_position,
                        button: egui_button,
                        pressed: true,
                        modifiers: egui_modifiers,
                    });
                    self.modifiers = *modifiers;
                }
            }
            InputEvent::MouseButtonRelease { button, position, modifiers } => {
                if let Some(egui_button) = map_mouse_button(*button) {
                    let egui_position = Pos2::new(position.x as f32, position.y as f32);
                    let egui_modifiers = map_modifiers(*modifiers);

                    self.collected_events.push(Event::PointerButton {
                        pos: egui_position,
                        button: egui_button,
                        pressed: false,
                        modifiers: egui_modifiers,
                    });
                    self.modifiers = *modifiers;
                }
            }
            InputEvent::MouseWheelRotated { direction, modifiers } => {
                self.collected_events.push(Event::Scroll(match direction {
                    MouseWheelDirection::Up => egui::Vec2::new(0.0, 20.0),
                    MouseWheelDirection::Down => egui::Vec2::new(0.0, -20.0),
                    MouseWheelDirection::Unknown => egui::Vec2::new(0.0, 0.0),
                }));
                self.modifiers = *modifiers;
            }
            InputEvent::KeyPress { key, repeat, modifiers } => {
                if let Some(egui_key) = map_key(*key) {
                    let egui_modifiers = map_modifiers(*modifiers);

                    self.collected_events.push(Event::Key { key: egui_key, pressed: true, repeat: *repeat, modifiers: egui_modifiers });
                    self.modifiers = *modifiers;
                }
            }
            InputEvent::KeyRelease { key, modifiers } => {
                if let Some(egui_key) = map_key(*key) {
                    let egui_modifiers = map_modifiers(*modifiers);

                    self.collected_events.push(Event::Key { key: egui_key, pressed: false, repeat: false, modifiers: egui_modifiers });
                    self.modifiers = *modifiers;
                }
            }
            InputEvent::CharPress { character, .. } => {
                if !character.is_ascii_control() {
                    self.collected_events.push(Event::Text(character.to_string()));
                }
            }
            _ => {}
        }
    }

    pub fn get_input(&mut self) -> RawInput {
        let input = RawInput {
            screen_rect: Some(Rect::from_two_pos(Pos2::new(0.0, 0.0), Pos2::new(self.screen_size.x, self.screen_size.y))),
            events: self.collected_events.clone(),
            max_texture_side: Some(self.max_texture_size as usize),
            modifiers: map_modifiers(self.modifiers),
            time: Some(self.time.elapsed().as_secs_f64()),
            ..Default::default()
        };
        self.collected_events.clear();

        input
    }

    pub fn draw(&mut self, renderer: &mut RendererContext, output: FullOutput) -> Result<()> {
        renderer.activate_camera(self.camera_id)?;

        for (id, delta) in output.textures_delta.set {
            let position = delta.pos.map(|pos| Vec2::new(pos[0] as f32, pos[1] as f32));

            match delta.image {
                ImageData::Font(font) => {
                    let data = font.srgba_pixels(None).flat_map(|a| a.to_array()).collect::<Vec<u8>>();
                    let size = Vec2::new(font.size[0] as f32, font.size[1] as f32);
                    self.update_texture(id, renderer, &data, position, size, delta.options, true)?;
                }
                ImageData::Color(image) => {
                    let pixels_ptr = image.pixels.as_ptr() as *const u8;
                    let data = unsafe { slice::from_raw_parts(pixels_ptr, image.pixels.len() * 4) };
                    let size = Vec2::new(image.size[0] as f32, image.size[1] as f32);
                    self.update_texture(id, renderer, data, position, size, delta.options, false)?;
                }
            };
        }

        for shape in self.inner.tessellate(output.shapes) {
            if let Primitive::Mesh(mesh) = shape.primitive {
                let mut vertices = Vec::new();
                for vertice in mesh.vertices {
                    vertices.push(vertice.pos.x);
                    vertices.push(vertice.pos.y);
                    vertices.push(vertice.color.r() as f32 / 255.0);
                    vertices.push(vertice.color.g() as f32 / 255.0);
                    vertices.push(vertice.color.b() as f32 / 255.0);
                    vertices.push(vertice.color.a() as f32 / 255.0);
                    vertices.push(vertice.uv.x);
                    vertices.push(vertice.uv.y);
                }

                let mut sprite = Sprite::new();
                sprite.shape = Shape::Custom(ShapeData::new(vertices, mesh.indices));
                sprite.texture_id = *self.textures.get(&mesh.texture_id).unwrap();

                let scissor_position = Vec2::new(shape.clip_rect.left(), renderer.viewport_size.y - shape.clip_rect.height() - shape.clip_rect.top());
                let scissor_size = Vec2::new(shape.clip_rect.width(), shape.clip_rect.height());

                renderer.enable_scissor(scissor_position, scissor_size);
                renderer.draw(&sprite)?;
                renderer.flush()?;
            }
        }

        renderer.disable_scissor();

        for id in output.textures_delta.free {
            let texture_id = self.textures.get(&id).unwrap();
            renderer.textures.remove(*texture_id)?;
        }

        Ok(())
    }

    fn update_texture(
        &mut self,
        id: TextureId,
        renderer: &mut RendererContext,
        data: &[u8],
        position: Option<Vec2>,
        size: Vec2,
        options: TextureOptions,
        font: bool,
    ) -> Result<()> {
        let texture_id = if let Some(position) = position {
            let texture_id = self.textures.get(&id).unwrap();
            let texture = renderer.textures.get(*texture_id)?;
            texture.update(Vec2::new(position[0], position[1]), size, data);

            *texture_id
        } else {
            let raw = RawTexture::new("", "", size, data);
            let texture_id = renderer.textures.store(Texture::new(renderer, &raw));
            self.textures.insert(id, texture_id);

            texture_id
        };

        if !font {
            let minification = match options.minification {
                egui::TextureFilter::Linear => TextureFilter::Linear,
                egui::TextureFilter::Nearest => TextureFilter::Nearest,
            };

            let magnification = match options.magnification {
                egui::TextureFilter::Linear => TextureFilter::Linear,
                egui::TextureFilter::Nearest => TextureFilter::Nearest,
            };

            renderer.textures.get(texture_id)?.set_filters(minification, magnification);
        }

        Ok(())
    }
}

fn map_key(key: Key) -> Option<egui::Key> {
    match key {
        Key::Enter => Some(egui::Key::Enter),
        Key::Escape => Some(egui::Key::Escape),
        Key::Backspace => Some(egui::Key::Backspace),
        Key::Space => Some(egui::Key::Space),
        Key::Control => None,
        Key::Shift => None,
        Key::Alt => None,

        Key::ArrowLeft => Some(egui::Key::ArrowLeft),
        Key::ArrowUp => Some(egui::Key::ArrowUp),
        Key::ArrowRight => Some(egui::Key::ArrowRight),
        Key::ArrowDown => Some(egui::Key::ArrowDown),

        Key::Key0 => Some(egui::Key::Num0),
        Key::Key1 => Some(egui::Key::Num1),
        Key::Key2 => Some(egui::Key::Num2),
        Key::Key3 => Some(egui::Key::Num3),
        Key::Key4 => Some(egui::Key::Num4),
        Key::Key5 => Some(egui::Key::Num5),
        Key::Key6 => Some(egui::Key::Num6),
        Key::Key7 => Some(egui::Key::Num7),
        Key::Key8 => Some(egui::Key::Num8),
        Key::Key9 => Some(egui::Key::Num9),

        Key::F1 => Some(egui::Key::F1),
        Key::F2 => Some(egui::Key::F2),
        Key::F3 => Some(egui::Key::F3),
        Key::F4 => Some(egui::Key::F4),
        Key::F5 => Some(egui::Key::F5),
        Key::F6 => Some(egui::Key::F6),
        Key::F7 => Some(egui::Key::F7),
        Key::F8 => Some(egui::Key::F8),
        Key::F9 => Some(egui::Key::F9),
        Key::F10 => Some(egui::Key::F10),
        Key::F11 => Some(egui::Key::F11),
        Key::F12 => Some(egui::Key::F12),

        Key::KeyA => Some(egui::Key::A),
        Key::KeyB => Some(egui::Key::B),
        Key::KeyC => Some(egui::Key::C),
        Key::KeyD => Some(egui::Key::D),
        Key::KeyE => Some(egui::Key::E),
        Key::KeyF => Some(egui::Key::F),
        Key::KeyG => Some(egui::Key::G),
        Key::KeyH => Some(egui::Key::H),
        Key::KeyI => Some(egui::Key::I),
        Key::KeyJ => Some(egui::Key::J),
        Key::KeyK => Some(egui::Key::K),
        Key::KeyL => Some(egui::Key::L),
        Key::KeyM => Some(egui::Key::M),
        Key::KeyN => Some(egui::Key::N),
        Key::KeyO => Some(egui::Key::O),
        Key::KeyP => Some(egui::Key::P),
        Key::KeyQ => Some(egui::Key::Q),
        Key::KeyR => Some(egui::Key::R),
        Key::KeyS => Some(egui::Key::S),
        Key::KeyT => Some(egui::Key::T),
        Key::KeyU => Some(egui::Key::U),
        Key::KeyV => Some(egui::Key::V),
        Key::KeyW => Some(egui::Key::W),
        Key::KeyX => Some(egui::Key::X),
        Key::KeyY => Some(egui::Key::Y),
        Key::KeyZ => Some(egui::Key::Z),

        Key::Num0 => Some(egui::Key::Num0),
        Key::Num1 => Some(egui::Key::Num1),
        Key::Num2 => Some(egui::Key::Num2),
        Key::Num3 => Some(egui::Key::Num3),
        Key::Num4 => Some(egui::Key::Num4),
        Key::Num5 => Some(egui::Key::Num5),
        Key::Num6 => Some(egui::Key::Num6),
        Key::Num7 => Some(egui::Key::Num7),
        Key::Num8 => Some(egui::Key::Num8),
        Key::Num9 => Some(egui::Key::Num9),

        Key::Unknown => None,
    }
}

fn map_mouse_button(button: MouseButton) -> Option<PointerButton> {
    match button {
        MouseButton::Left => Some(PointerButton::Primary),
        MouseButton::Middle => Some(PointerButton::Middle),
        MouseButton::Right => Some(PointerButton::Secondary),
        MouseButton::Unknown => None,
    }
}

fn map_modifiers(modifiers: Modifiers) -> egui::Modifiers {
    egui::Modifiers { ctrl: modifiers.control, alt: modifiers.alt, shift: modifiers.shift, command: false, mac_cmd: false }
}
