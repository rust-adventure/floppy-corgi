use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod background_material;
pub mod pipes;

pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
pub const CORGI_SIZE: f32 = 25.0;

#[derive(AssetCollection, Resource)]
pub struct MyAssets {
    #[asset(texture_atlas_layout(
        tile_size_x = 500,
        tile_size_y = 500,
        columns = 12,
        rows = 1
    ))]
    pub corgi_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "corgi.png")]
    pub corgi: Handle<Image>,

    #[asset(path = "hill_large.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub hill: Handle<Image>,

    #[asset(path = "background_color_grass.png")]
    #[asset(image(sampler(filter = nearest, wrap = repeat)))]
    pub background: Handle<Image>,
}
