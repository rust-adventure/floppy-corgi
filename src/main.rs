use bevy::{
    prelude::*, render::texture::ImageSettings,
    sprite::Anchor,
};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use floppy_corgi::pipes::{Pipe, PointsGate, SpawnPipe};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Floppy Corgi".to_string(),
            width: 1200.0,
            height: 600.0,
            ..Default::default()
        })
        .init_resource::<Score>()
        .insert_resource(ClearColor(Color::rgb(0.0, 42.0/255.0, 0.0)))
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .init_resource::<NumPipesToSpawn>()
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
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_system_set(SystemSet::on_update(MyStates::Next)
            .with_system(animate_sprite)
            .with_system(corgi_control)
            .with_system(align_to_window)
            .with_system(display_events)
            .with_system(despawn_pipes)
        )
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
    #[asset(path = "hill_large.png")]
    hill: Handle<Image>,
    #[asset(path = "backgroundColorGrass.png")]
    background: Handle<Image>,
}

#[derive(Component)]
struct Corgi;

#[derive(Component)]
struct Ground;

struct NumPipesToSpawn(u32);

#[derive(Default)]
struct Score(u32);

impl FromWorld for NumPipesToSpawn {
    fn from_world(world: &mut World) -> Self {
        let window = world
            .get_resource::<Windows>()
            .unwrap()
            .primary();

        let num_pipes = (window.width() / 400.0) as u32;

        NumPipesToSpawn(num_pipes + 1)
    }
}

fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
    assets: Res<MyAssets>,
    num_pipes: Res<NumPipesToSpawn>,
) {
    let window = windows.primary();

    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.0;
    commands.spawn_bundle(camera);

    let sprite_size = 100.0;

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..Default::default()
        },
        texture: assets.background.clone(),
        ..Default::default()
    });

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
                color: Color::rgb(0.14, 0.75, 0.46),
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

    for index in 0..(num_pipes.0) {
        commands.add(SpawnPipe {
            image: assets.hill.clone(),
            transform: Transform::from_xyz(
                200.0 + 400.0 * index as f32,
                0.0,
                1.0,
            ),
        });
    }
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
    mut corgi: Query<
        (&mut Velocity, &mut ExternalImpulse),
        With<Corgi>,
    >,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        let (mut velocity, mut impulse) =
            corgi.single_mut();
        impulse.impulse = Vect::new(0.0, 200.0);
        velocity.linvel = Vec2::new(0.0, 0.0);
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
    gates: Query<Entity, With<PointsGate>>,
    corgi: Query<Entity, With<Corgi>>,
    mut score: ResMut<Score>,
) {
    for collision_event in collision_events.iter() {
        let corgi = corgi.single();
        match collision_event {
            CollisionEvent::Started(a, b, _flags) => {
                if let Some((_corgi, other)) =
                    if corgi == *a {
                        Some((a, b))
                    } else if corgi == *b {
                        Some((b, a))
                    } else {
                        None
                    }
                {
                    if let Ok(_gate_entity) =
                        gates.get(*other)
                    {
                        score.0 += 1;
                    }
                }
            }
            CollisionEvent::Stopped(_a, _b, _flags) => {}
        }
    }
}

fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>,
    windows: Res<Windows>,
    assets: Res<MyAssets>,
) {
    let window = windows.primary();
    for (entity, transform) in pipes.iter() {
        if transform.translation.x < -window.width() / 2.0 {
            commands.entity(entity).despawn_recursive();

            commands.add(SpawnPipe {
                image: assets.hill.clone(),
                transform: Transform::from_xyz(
                    window.width() - 200.0,
                    0.0,
                    1.0,
                ),
            });
        }
    }
}
