use orion_core::app::ApplicationContext;
use orion_core::fast_gpu;
use orion_core::window::Coordinates;
use orion_core::window::WindowStyle;
use scenes::boot::BootScene;
use scenes::game::GameScene;
use scenes::loading::LoadingScene;
use scenes::menu::MenuScene;
use scenes::GlobalData;

pub mod scenes;
pub mod ui;

fast_gpu!();

fn main() {
    ApplicationContext::<GlobalData>::new("Template", WindowStyle::Window { size: Coordinates::new(800, 600) })
        .unwrap()
        .with_scene("BootScene", Box::<BootScene>::default())
        .with_scene("LoadingScene", Box::<LoadingScene>::default())
        .with_scene("MenuScene", Box::<MenuScene>::default())
        .with_scene("GameScene", Box::<GameScene>::default())
        .run("BootScene")
        .unwrap();
}
