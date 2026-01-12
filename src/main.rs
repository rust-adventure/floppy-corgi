use bevy::color::palettes::tailwind::RED_400;
use bevy::{
    camera::ScalingMode,
    math::bounding::{
        Aabb2d, BoundingCircle, IntersectsVolume,
    },
    prelude::*,
};
use flappy_bird::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PipePlugin)
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                gravity,
                check_in_bounds,
                check_collisions,
            )
                .chain(),
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
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = config_store
        .config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = true;

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

fn check_collisions(
    mut commands: Commands,
    player: Single<(&Sprite, Entity), With<Player>>,
    pipe_segments: Query<
        (&Sprite, Entity),
        Or<(With<PipeTop>, With<PipeBottom>)>,
    >,
    pipe_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    mut gizmos: Gizmos,
    transform_helper: TransformHelper,
) -> Result<()> {
    let player_transform = transform_helper
        .compute_global_transform(player.1)?;
    let player_collider = BoundingCircle::new(
        player_transform.translation().xy(),
        PLAYER_SIZE / 2.,
    );

    gizmos.circle_2d(
        player_transform.translation().xy(),
        PLAYER_SIZE / 2.,
        RED_400,
    );

    for (sprite, entity) in &pipe_segments {
        let pipe_transform = transform_helper
            .compute_global_transform(entity)?;
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        gizmos.rect_2d(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap(),
            RED_400,
        );
        if player_collider.intersects(&pipe_collider) {
            commands.trigger(EndGame);
        }
    }

    for (sprite, entity) in &pipe_gaps {
        let gap_transform = transform_helper
            .compute_global_transform(entity)?;
        let gap_collider = Aabb2d::new(
            gap_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        gizmos.rect_2d(
            gap_transform.translation().xy(),
            sprite.custom_size.unwrap().xy(),
            RED_400,
        );

        if player_collider.intersects(&gap_collider) {
            info!("score a point!");
            commands.entity(entity).despawn();
        }
    }

    Ok(())
}
