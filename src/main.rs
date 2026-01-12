use bevy::{camera::ScalingMode, prelude::*};
use flappy_bird::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PipePlugin)
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (gravity, check_in_bounds),
        )
        .add_systems(Update, controls)
        .add_observer(respawn_on_endgame)
        .run()
}

#[derive(Component)]
#[require(Gravity(1000.), Velocity)]
struct Player;

#[derive(Component)]
struct Gravity(f32);

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Event)]
struct EndGame;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: CANVAS_SIZE.x,
                max_height: CANVAS_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Player,
        Sprite {
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            image: asset_server.load("bevy-bird.png"),
            ..default()
        },
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
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

fn controls(
    mut velocity: Single<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        velocity.0 = 400.;
    }
}

fn check_in_bounds(
    player: Single<&Transform, With<Player>>,
    mut commands: Commands,
) {
    if player.translation.y
        < -CANVAS_SIZE.y / 2.0 - PLAYER_SIZE
        || player.translation.y
            > CANVAS_SIZE.y / 2.0 + PLAYER_SIZE
    {
        commands.trigger(EndGame);
    }
}

fn respawn_on_endgame(
    _: On<EndGame>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    commands.entity(*player).insert((
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
        Velocity(0.),
    ));
}
