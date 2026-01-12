use bevy::{image::ImageLoaderSettings, prelude::*};

pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
pub const PLAYER_SIZE: f32 = 25.0;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            spawn_pipes.run_if(run_once),
        );
    }
}

fn spawn_pipes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let image = asset_server.load_with_settings(
        "pipe.png",
        |settings: &mut ImageLoaderSettings| {
            settings
                .sampler
                .get_or_init_descriptor()
                .set_filter(
                    bevy::image::ImageFilterMode::Nearest,
                );
        },
    );
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(Vec2::new(32., 160.)),
            image_mode: SpriteImageMode::Sliced(
                TextureSlicer {
                    border: BorderRect::axes(8., 19.),
                    center_scale_mode:
                        SliceScaleMode::Stretch,
                    ..default()
                },
            ),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}
