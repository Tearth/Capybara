use crate::assets::loader::AssetsLoader;
use crate::audio::context::AudioContext;
use crate::renderer::context::RendererContext;
use crate::scene::FrameCommand;
use crate::scene::Scene;
use crate::ui::context::UiContext;
use crate::window::InputEvent;
use crate::window::WindowContext;
use crate::window::WindowStyle;
use anyhow::Result;
use glam::Vec2;
use instant::Instant;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct ApplicationContext {
    pub window: Box<WindowContext>,
    pub renderer: RendererContext,
    pub assets: AssetsLoader,
    pub ui: UiContext,
    pub audio: AudioContext,
    pub scenes: HashMap<String, Box<dyn Scene>>,

    current_scene: String,
    next_scene: Option<String>,
    frame_timestamp: Instant,
    running: bool,
}

pub struct ApplicationState<'a> {
    pub window: &'a mut Box<WindowContext>,
    pub renderer: &'a mut RendererContext,
    pub assets: &'a mut AssetsLoader,
    pub ui: &'a mut UiContext,
    pub audio: &'a mut AudioContext,
}

impl ApplicationContext {
    pub fn new(title: &str, style: WindowStyle) -> Result<Self> {
        let window = WindowContext::new(title, style)?;
        let mut renderer = RendererContext::new(window.load_gl_pointers())?;
        let assets = AssetsLoader::new();
        let ui = UiContext::new(&mut renderer)?;
        let audio = AudioContext::new()?;

        Ok(Self {
            window,
            renderer,
            assets,
            ui,
            audio,
            scenes: Default::default(),
            current_scene: "".to_string(),
            next_scene: None,
            frame_timestamp: Instant::now(),
            running: true,
        })
    }

    pub fn with_scene(mut self, name: &str, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(name.to_string(), scene);
        self
    }

    pub fn run(self, scene: &str) -> Result<()> {
        let app = Rc::new(RefCell::new(self));

        #[cfg(web)]
        {
            let app_clone = app.clone();
            app.borrow_mut().window.init_closures(app.clone(), move || app_clone.borrow_mut().run_internal().unwrap());
        }

        app.borrow_mut().next_scene = Some(scene.to_string());

        #[cfg(any(windows, unix))]
        app.borrow_mut().run_internal()?;

        Ok(())
    }

    pub fn run_internal(&mut self) -> Result<()> {
        self.window.set_swap_interval(1);

        while self.running {
            if let Some(next_scene) = &self.next_scene {
                if !self.current_scene.is_empty() {
                    let old_scene = self.scenes.get_mut(&self.current_scene).unwrap();
                    old_scene.deactivation(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets, &mut self.ui, &mut self.audio))?;
                }

                let new_scene = self.scenes.get_mut(next_scene).unwrap();
                new_scene.activation(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets, &mut self.ui, &mut self.audio))?;

                self.current_scene = next_scene.clone();
                self.next_scene = None;
            }

            let scene = self.scenes.get_mut(&self.current_scene).unwrap();

            while let Some(event) = self.window.poll_event() {
                match event {
                    InputEvent::WindowSizeChange { size } => {
                        self.renderer.set_viewport(Vec2::new(size.x as f32, size.y as f32))?;
                    }
                    InputEvent::WindowClose => {
                        return Ok(());
                    }
                    _ => {}
                }

                self.ui.collect_event(&event);
                scene.input(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets, &mut self.ui, &mut self.audio), event)?;
            }

            let ui_input = self.ui.get_input();
            let ui_output = scene.ui(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets, &mut self.ui, &mut self.audio), ui_input)?;

            let now = Instant::now();
            let delta = (now - self.frame_timestamp).as_secs_f32();
            self.frame_timestamp = now;

            self.renderer.begin_user_frame()?;
            match scene.frame(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets, &mut self.ui, &mut self.audio), delta)? {
                Some(FrameCommand::ChangeScene { name }) => self.next_scene = Some(name),
                Some(FrameCommand::Exit) => self.running = false,
                None => {}
            }
            self.renderer.end_user_frame()?;

            self.ui.draw(&mut self.renderer, ui_output)?;
            self.window.swap_buffers();

            #[cfg(web)]
            return Ok(());
        }

        Ok(())
    }
}

impl<'a> ApplicationState<'a> {
    pub fn new(
        window: &'a mut Box<WindowContext>,
        renderer: &'a mut RendererContext,
        assets: &'a mut AssetsLoader,
        ui: &'a mut UiContext,
        audio: &'a mut AudioContext,
    ) -> Self {
        Self { window, renderer, assets, ui, audio }
    }
}
