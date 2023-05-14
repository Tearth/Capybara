use crate::assets::loader::AssetsLoader;
use crate::renderer::context::RendererContext;
use crate::ui::context::UiContext;
use crate::user::UserSpace;
use crate::window::InputEvent;
use crate::window::WindowContext;
use crate::window::WindowStyle;
use anyhow::Result;
use glam::Vec2;
use instant::Instant;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ApplicationContext<U> {
    pub window: Box<WindowContext>,
    pub renderer: RendererContext,
    pub assets: AssetsLoader,
    pub ui: UiContext,
    pub user: U,

    frame_timestamp: Instant,
}

pub struct ApplicationState<'a> {
    pub window: &'a mut Box<WindowContext>,
    pub renderer: &'a mut RendererContext,
    pub assets: &'a mut AssetsLoader,
}

impl<U> ApplicationContext<U>
where
    U: UserSpace + 'static,
{
    pub fn new(user: U, title: &str, style: WindowStyle) -> Result<Self> {
        let window = WindowContext::new(title, style)?;
        let mut renderer = RendererContext::new(window.load_gl_pointers())?;
        let assets = AssetsLoader::new();
        let ui = UiContext::new(&mut renderer)?;

        Ok(Self { window, renderer, assets, ui, user, frame_timestamp: Instant::now() })
    }

    pub fn run(self) -> Result<()> {
        let app = Rc::new(RefCell::new(self));

        #[cfg(web)]
        {
            let app_clone = app.clone();
            app.borrow_mut().window.init_closures(app.clone(), move || app_clone.borrow_mut().run_internal().unwrap());
        }

        #[cfg(any(windows, unix))]
        app.borrow_mut().run_internal()?;

        Ok(())
    }

    pub fn run_internal(&mut self) -> Result<()> {
        self.window.set_swap_interval(1);

        loop {
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
                self.user.input(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets), event);
            }

            let ui_input = self.ui.get_input();
            let ui_output = self.ui.inner.run(ui_input, |context| {
                self.user.ui(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets), context);
            });

            let now = Instant::now();
            let delta = (now - self.frame_timestamp).as_secs_f32();
            self.frame_timestamp = now;

            self.renderer.begin_user_frame()?;
            self.user.frame(ApplicationState::new(&mut self.window, &mut self.renderer, &mut self.assets), delta);
            self.renderer.end_user_frame();

            self.ui.draw(&mut self.renderer, ui_output)?;
            self.window.swap_buffers();

            #[cfg(web)]
            return Ok(());
        }
    }
}

impl<'a> ApplicationState<'a> {
    pub fn new(window: &'a mut Box<WindowContext>, renderer: &'a mut RendererContext, assets: &'a mut AssetsLoader) -> Self {
        Self { window, renderer, assets }
    }
}
