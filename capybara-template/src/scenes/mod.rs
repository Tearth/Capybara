use capybara_core::assets::loader::AssetsLoader;

pub mod boot;
pub mod game;
pub mod loading;
pub mod menu;

#[derive(Default)]
pub struct GlobalData {
    assets: AssetsLoader,
}
