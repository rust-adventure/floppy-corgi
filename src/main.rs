use bevy::{prelude::*, sprite::Anchor};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use noise::{BasicMulti, NoiseFn};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Floppy Corgi".to_string(),
            width: 1200.0,
            height: 600.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(setup),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(animate_sprite)
        .add_system(corgi_control)
        .add_system(align_to_window)
        .add_system(display_events)
        .add_system(display_intersection_info)
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(texture_atlas(
        tile_size_x = 500.,
        tile_size_y = 500.,
        columns = 12,
        rows = 1
    ))]
    #[asset(path = "corgi.png")]
    corgi: Handle<TextureAtlas>,
}

#[derive(Component)]
struct Corgi;

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct PipeDespawnArea;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct PipeTop;

#[derive(Component)]
struct PipeBottom;

#[derive(Component)]
struct PointsGate;

fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
    assets: Res<MyAssets>,
) {
    let window = windows.primary();

    commands.spawn_bundle(Camera2dBundle::default());

    let sprite_size = 100.0;

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: assets.corgi.clone(),
            transform: Transform::from_xyz(
                -window.width() / 4.0,
                0.0,
                1.0,
            ),
            sprite: TextureAtlasSprite {
                flip_x: true,
                custom_size: Some(Vec2::new(
                    sprite_size,
                    sprite_size,
                )),
                ..default()
            },
            ..default()
        })
        .insert(AnimationTimer(Timer::from_seconds(
            0.1, true,
        )))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Collider::cuboid(
            sprite_size / 2.0,
            sprite_size / 2.0,
        ))
        // .insert(ColliderMassProperties::Mass(20.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            torque_impulse: 0.0,
        })
        .insert(GravityScale(2.5))
        .insert(Corgi)
        .insert(ActiveEvents::COLLISION_EVENTS);

    // Ground
    let ground_size =
        Vec2::new(window.width(), window.height() / 10.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GREEN,
                custom_size: Some(Vec2::new(
                    ground_size.x,
                    ground_size.y,
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                0.0,
                -window.height() / 2.0
                    + ground_size.y / 2.0,
                1.0,
            ),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            ground_size.x / 2.0,
            ground_size.y / 2.0,
        ))
        .insert(Ground);

    // pipe despawn area
    let pipe_despawn_area =
        Vec2::new(window.width() / 20.0, window.height());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(
                    pipe_despawn_area.x,
                    pipe_despawn_area.y,
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                -window.width() / 2.0
                    - pipe_despawn_area.x / 2.0,
                0.0,
                1.0,
            ),
            ..default()
        })
        .insert(Sensor)
        .insert(Collider::cuboid(
            pipe_despawn_area.x / 2.0,
            pipe_despawn_area.y / 2.0,
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(PipeDespawnArea);

    // Pipe
    let pipe_size = Vec2::new(200.0, window.height() / 2.0);
    let noise = BasicMulti::new();
    let center_of_opening = noise.get([1.0 / 0.02, 0.0]);
    let gap_size = 300.0;

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(200.0, 0.0, 1.0),
            visibility: Visibility { is_visible: true },
            ..default()
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(Velocity::linear(Vec2::new(-100.0, 0.0)))
        .with_children(|builder| {
            // top pipe
            builder
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::SEA_GREEN,
                        custom_size: Some(Vec2::new(
                            pipe_size.x,
                            pipe_size.y,
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        0.0,
                        pipe_size.y / 2.0 + gap_size / 2.0,
                        1.0,
                    ),
                    ..default()
                })
                .insert(Collider::cuboid(
                    pipe_size.x / 2.0,
                    pipe_size.y / 2.0,
                ))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(PipeTop);

            // Gap Sensor
            builder
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        custom_size: Some(Vec2::new(
                            10.0, gap_size,
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        0.0, 0.0, 1.0,
                    ),
                    ..default()
                })
                .insert(Sensor)
                .insert(Collider::cuboid(
                    5.0,
                    gap_size / 2.0,
                ))
                .insert(PointsGate)
                .insert(ActiveEvents::COLLISION_EVENTS);

            // bottom pipe
            builder
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::SEA_GREEN,
                        custom_size: Some(Vec2::new(
                            pipe_size.x,
                            pipe_size.y,
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        0.0,
                        -pipe_size.y / 2.0 - gap_size / 2.0,
                        1.0,
                    ),
                    ..default()
                })
                .insert(Collider::cuboid(
                    pipe_size.x / 2.0,
                    pipe_size.y / 2.0,
                ))
                .insert(PipeBottom);
        })
        .insert(Pipe);
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in
        &mut query
    {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases
                .get(texture_atlas_handle)
                .unwrap();
            sprite.index = (sprite.index + 1)
                % texture_atlas.textures.len();
        }
    }
}

fn corgi_control(
    mut corgi: Query<&mut ExternalImpulse, With<Corgi>>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        corgi.single_mut().impulse = Vect::new(0.0, 200.0);
    }
}

fn align_to_window(
    windows: Res<Windows>,
    mut corgis: Query<&mut Transform, With<Corgi>>,
    mut ground: Query<
        (
            &mut Sprite,
            &mut Transform,
            &mut Collider,
        ),
        (With<Ground>, Without<Corgi>),
    >,
) {
    if !windows.is_changed() {
        return;
    }

    let window = windows.primary();
    for mut corgi in corgis.iter_mut() {
        corgi.translation.x = -window.width() / 4.0;
    }

    for (mut sprite, mut transform, mut collider) in
        ground.iter_mut()
    {
        let ground_size = Vec2::new(
            window.width(),
            window.height() / 10.0,
        );
        sprite.custom_size =
            Some(Vec2::new(ground_size.x, ground_size.y));

        transform.translation.y =
            -window.height() / 2.0 + ground_size.y / 2.0;
        *collider = Collider::cuboid(
            ground_size.x / 2.0,
            ground_size.y / 2.0,
        )
    }
}

/* A system that displays the events. */
fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        println!(
            "Received collision event: {:?}",
            collision_event
        );
    }
}

fn display_intersection_info(
    mut commands: Commands,
    mut gates: Query<
        (Entity, &Parent, &mut Transform),
        (With<PointsGate>, Without<Pipe>),
    >,
    mut pipes: Query<&mut Transform, With<Pipe>>,
    pipe_despawn_area: Query<Entity, With<PipeDespawnArea>>,
    rapier_context: Res<RapierContext>,
) {
    for mut gate in gates.iter_mut() {
        let despawn = pipe_despawn_area.single();

        if let Some(value) = rapier_context
            .intersection_pair(gate.0, despawn)
        {
            dbg!(value);
            println!("The entities {:?} and {:?} have intersecting colliders!",gate.0, despawn);
            let mut transform =
                pipes.get_mut(gate.1.get()).unwrap();
            transform.translation.x = 500.0;
        }
    }
}
