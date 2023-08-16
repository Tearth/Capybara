use crate::renderer::context::RendererContext;
use crate::scene::FrameCommand;
use crate::scene::Scene;
use crate::ui::context::UiContext;
use crate::utils::storage::Storage;
use crate::window::InputEvent;
use crate::window::WindowContext;
use crate::window::WindowStyle;
use anyhow::Result;
use glam::Vec2;
use instant::Instant;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "audio")]
use crate::audio::context::AudioContext;

#[cfg(feature = "physics")]
use crate::physics::context::PhysicsContext;

pub struct ApplicationContext<G>
where
    G: Default + 'static,
{
    pub window: Box<WindowContext>,
    pub renderer: RendererContext,
    pub ui: UiContext,
    pub scenes: Storage<Box<dyn Scene<G>>>,
    pub global: G,

    #[cfg(feature = "audio")]
    pub audio: AudioContext,

    #[cfg(feature = "physics")]
    pub physics: PhysicsContext,

    current_scene: String,
    next_scene: Option<String>,
    frame_timestamp: Instant,
    running: bool,
    timestep: f32,
    accumulator: f32,
}

pub struct ApplicationState<'a, G> {
    pub window: &'a mut Box<WindowContext>,
    pub renderer: &'a mut RendererContext,
    pub ui: &'a mut UiContext,
    pub global: &'a mut G,

    #[cfg(feature = "audio")]
    pub audio: &'a mut AudioContext,

    #[cfg(feature = "physics")]
    pub physics: &'a mut PhysicsContext,
}

macro_rules! state {
    ($self:ident) => {
        ApplicationState {
            window: &mut $self.window,
            renderer: &mut $self.renderer,
            ui: &mut $self.ui,
            global: &mut $self.global,

            #[cfg(feature = "audio")]
            audio: &mut $self.audio,

            #[cfg(feature = "physics")]
            physics: &mut $self.physics,
        }
    };
}

impl<G> ApplicationContext<G>
where
    G: Default + 'static,
{
    pub fn new(title: &str, style: WindowStyle) -> Result<Self> {
        let window = WindowContext::new(title, style)?;
        let mut renderer = RendererContext::new(window.load_gl_pointers())?;
        let ui = UiContext::new(&mut renderer)?;

        #[cfg(feature = "audio")]
        let audio = AudioContext::new()?;

        #[cfg(feature = "physics")]
        let physics = PhysicsContext::new();

        Ok(Self {
            window,
            renderer,
            ui,
            scenes: Default::default(),
            global: Default::default(),

            #[cfg(feature = "audio")]
            audio,

            #[cfg(feature = "physics")]
            physics,

            current_scene: "".to_string(),
            next_scene: None,
            frame_timestamp: Instant::now(),
            running: true,
            timestep: 1.0 / 60.0,
            accumulator: 0.0,
        })
    }

    pub fn with_scene(mut self, name: &str, scene: Box<dyn Scene<G>>) -> Result<Self> {
        self.scenes.store_with_name(name, scene)?;
        Ok(self)
    }

    pub fn run(self, scene: &str) -> Result<()> {
        let app = Rc::new(RefCell::new(self));

        #[cfg(web)]
        {
            let app_clone = app.clone();
            app.borrow_mut().window.init_closures(app.clone(), move || app_clone.borrow_mut().run_internal().unwrap());
        }

        app.borrow_mut().next_scene = Some(scene.to_string());
        app.borrow_mut().window.set_swap_interval(1);
        app.borrow_mut().run_internal()?;

        Ok(())
    }

    pub fn run_internal(&mut self) -> Result<()> {
        while self.running {
            self.renderer.begin_frame()?;

            if let Some(next_scene) = &self.next_scene {
                if !self.current_scene.is_empty() {
                    self.scenes.get_by_name_mut(&self.current_scene)?.deactivation(state!(self))?;
                }

                self.scenes.get_by_name_mut(next_scene)?.activation(state!(self))?;
                self.current_scene = next_scene.clone();
                self.next_scene = None;
            }

            let scene = self.scenes.get_by_name_mut(&self.current_scene)?;

            while let Some(event) = self.window.poll_event() {
                match event {
                    InputEvent::WindowSizeChange { size } => self.renderer.set_viewport(Vec2::new(size.x as f32, size.y as f32))?,
                    InputEvent::WindowClose => return Ok(()),
                    _ => {}
                }

                self.ui.collect_event(&event);
                scene.input(state!(self), event)?;
            }

            let ui_input = self.ui.get_input();
            let (ui_output, command) = scene.ui(state!(self), ui_input)?;
            self.process_frame_command(command);

            let now = Instant::now();
            let mut delta = (now - self.frame_timestamp).as_secs_f32();

            if delta > 0.1 {
                delta = 0.1;
            }

            self.frame_timestamp = now;
            self.accumulator += delta;

            while self.accumulator >= self.timestep {
                #[cfg(feature = "physics")]
                self.physics.step(self.timestep);

                let scene = self.scenes.get_by_name_mut(&self.current_scene)?;
                let command = scene.fixed(state!(self))?;
                self.process_frame_command(command);

                self.accumulator -= self.timestep;
            }

            let scene = self.scenes.get_by_name_mut(&self.current_scene)?;
            let command = scene.frame(state!(self), self.accumulator, delta)?;
            self.process_frame_command(command);
            self.renderer.flush_buffer()?;

            self.ui.draw(&mut self.renderer, ui_output)?;

            self.renderer.end_frame()?;
            self.window.swap_buffers();

            #[cfg(web)]
            return Ok(());
        }

        Ok(())
    }

    fn process_frame_command(&mut self, command: Option<FrameCommand>) {
        match command {
            Some(FrameCommand::ChangeScene { name }) => self.next_scene = Some(name),
            Some(FrameCommand::Exit) => self.running = false,
            None => {}
        }
    }
}
