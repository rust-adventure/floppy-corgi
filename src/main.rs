use bevy::{prelude::*, render::camera};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, corgi_control)
        .add_systems(FixedUpdate, gravity)
        .run()
}

#[derive(Component)]
#[require(Gravity(1000.), Velocity)]
struct Player;

#[derive(Component)]
struct Gravity(f32);

#[derive(Component, Default)]
struct Velocity(f32);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<
        Assets<TextureAtlasLayout>,
    >,
) {
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

    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(500),
        12,
        1,
        None,
        None,
    );
    let texture_atlas_layout =
        texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite {
            flip_x: true,
            custom_size: Some(Vec2::splat(25.)),
            image: asset_server.load("corgi.png"),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
    ));
}

fn gravity(
    mut transforms: Query<(
        &mut Transform,
        &mut Velocity,
        &Gravity,
    )>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, gravity) in
        &mut transforms
    {
        velocity.0 -= gravity.0 * time.delta_secs();

        transform.translation.y +=
            velocity.0 * time.delta_secs();
    }
}

fn corgi_control(
    mut corgi_velocity: Single<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        corgi_velocity.0 = 400.;
    }
}
