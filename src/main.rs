use bevy::{prelude::*, render::camera};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run()
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: camera::ScalingMode::AutoMax {
                max_width: 480.,
                max_height: 270.,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::splat(25.)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}
