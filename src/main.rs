use bevy::{prelude::*, render::camera};

pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
pub const CORGI_SIZE: f32 = 25.0;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, corgi_control)
        .add_systems(
            FixedUpdate,
            (gravity, check_in_bounds),
        )
        .add_observer(respawn_on_endgame)
        .run()
}

fn respawn_on_endgame(
    _trigger: Trigger<EndGame>,
    mut commands: Commands,
    corgi: Single<Entity, With<Player>>,
) {
    commands.entity(*corgi).insert((
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
        Velocity(0.),
    ));
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
                max_width: CANVAS_SIZE.x,
                max_height: CANVAS_SIZE.y,
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
            custom_size: Some(Vec2::splat(CORGI_SIZE)),
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

#[derive(Event)]
struct EndGame;

fn check_in_bounds(
    corgi: Single<&Transform, With<Player>>,
    mut commands: Commands,
) {
    if corgi.translation.y
        < -CANVAS_SIZE.y / 2.0 - CORGI_SIZE
        || corgi.translation.y
            > CANVAS_SIZE.y / 2.0 + CORGI_SIZE
    {
        commands.trigger(EndGame);
    }
}
