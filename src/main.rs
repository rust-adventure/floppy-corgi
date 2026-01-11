use bevy::{camera::ScalingMode, prelude::*};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run()
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: 480.,
                max_height: 270.,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::splat(25.)),
            image: asset_server.load("bevy-bird.png"),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}
