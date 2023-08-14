use capybara::app::ApplicationContext;
use capybara::fast_gpu;
use capybara::window::Coordinates;
use capybara::window::WindowStyle;
use scenes::boot::BootScene;
use scenes::game::GameScene;
use scenes::loading::LoadingScene;
use scenes::menu::MenuScene;
use scenes::GlobalData;

pub mod scenes;
pub mod ui;

fast_gpu!();

fn main() {
    ApplicationContext::<GlobalData>::new("Template", WindowStyle::Window { size: Coordinates::new(1280, 720) })
        .unwrap()
        .with_scene("BootScene", Box::<BootScene>::default())
        .with_scene("LoadingScene", Box::<LoadingScene>::default())
        .with_scene("MenuScene", Box::<MenuScene>::default())
        .with_scene("GameScene", Box::<GameScene>::default())
        .run("BootScene")
        .unwrap();
}