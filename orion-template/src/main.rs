use orion_core::app::ApplicationContext;
use orion_core::fast_gpu;
use orion_core::window::Coordinates;
use orion_core::window::WindowStyle;
use scenes::about::AboutScene;
use scenes::boot::BootScene;
use scenes::loading::LoadingScene;
use scenes::menu::MenuScene;

pub mod scenes;
pub mod ui;

fast_gpu!();

fn main() {
    ApplicationContext::new("Template", WindowStyle::Window { size: Coordinates::new(800, 600) })
        .unwrap()
        .with_scene("BootScene", Box::<BootScene>::default())
        .with_scene("LoadingScene", Box::<LoadingScene>::default())
        .with_scene("MenuScene", Box::<MenuScene>::default())
        .with_scene("AboutScene", Box::<AboutScene>::default())
        .run("BootScene")
        .unwrap();
}
