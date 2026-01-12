use bevy::{
    camera::ScalingMode,
    image::ImageAddressMode,
    math::bounding::{
        Aabb2d, BoundingCircle, IntersectsVolume,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};
use bevy::{
    color::palettes::tailwind::RED_400,
    image::ImageLoaderSettings,
};
use flappy_bird::*;

fn main() -> AppExit {
    App::new()
        .init_resource::<Score>()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            PipePlugin,
            Material2dPlugin::<BackgroundMaterial>::default(
            ),
        ))
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
        .add_systems(
            Update,
            (
                controls,
                score_update
                    .run_if(resource_changed::<Score>),
                enforce_bird_direction,
            ),
        )
        .add_observer(respawn_on_endgame)
        .add_observer(
            |_trigger: On<ScorePoint>,
             mut score: ResMut<Score>| {
                score.0 += 1;
            },
        )
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

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Event)]
pub struct ScorePoint;

#[derive(Component)]
struct ScoreText;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut config_store: ResMut<GizmoConfigStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let (config, _) = config_store
        .config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = false;

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
            color: Srgba::hex("#282828").unwrap().into(),
            ..default()
        },
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
    ));

    commands.spawn((
        Node {
            width: percent(100.),
            margin: px(20.).top(),
            ..default()
        },
        Text::new("0"),
        TextLayout::new_with_justify(Justify::Center),
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(Srgba::hex("#282828").unwrap().into()),
        ScoreText,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.x,
        ))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: asset_server.load_with_settings(
                "background_color_grass.png",
                |settings: &mut ImageLoaderSettings| {
                    settings
                        .sampler
                        .get_or_init_descriptor()
                        .set_address_mode(
                            ImageAddressMode::Repeat,
                        );
                },
            ),
        })),
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
    mut score: ResMut<Score>,
) {
    score.0 = 0;
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
            commands.trigger(ScorePoint);
            commands.entity(entity).despawn();
        }
    }

    Ok(())
}

fn score_update(
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut span in &mut query {
        span.0 = score.0.to_string();
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "background.wgsl".into()
    }
}

fn enforce_bird_direction(
    mut player: Single<
        (&mut Transform, &Velocity),
        With<Player>,
    >,
) {
    let calculated_velocity =
        Vec2::new(PIPE_SPEED, player.1.0);
    player.0.rotation = Quat::from_rotation_z(
        calculated_velocity.to_angle(),
    );
}
