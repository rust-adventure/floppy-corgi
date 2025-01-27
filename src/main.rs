use bevy::{prelude::*, window::PrimaryWindow};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

mod pipes;
use crate::pipes::{Pipe, PointsGate, SpawnPipe};

fn main() {
    let window = Window {
        title: "Floppy Corgi".to_string(),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        resolution: Vec2::new(1200.0, 600.0).into(),
        ..default()
    };
    let window_plugin = WindowPlugin {
        primary_window: Some(window),
        ..default()
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(ImagePlugin::default_nearest()),
        )
        .init_resource::<Score>()
        .insert_resource(ClearColor(Color::srgb(0.0, 42.0 / 255.0, 0.0)))
        .init_resource::<NumPipesToSpawn>()
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_systems(
            Update,
            (
                animate_sprite,
                corgi_control,
                align_to_window,
                display_events,
                despawn_pipes,
            )
                .run_if(in_state(MyStates::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 500, tile_size_y = 500, columns = 12, rows = 1))]
    #[asset(path = "corgi.png")]
    corgi: Handle<Image>,
    #[asset(path = "hill_large.png")]
    hill: Handle<Image>,
    #[asset(path = "backgroundColorGrass.png")]
    background: Handle<Image>,
}

#[derive(Component)]
struct Corgi;

#[derive(Component)]
struct Ground;

#[derive(Default, Resource)]
struct Score(u32);

#[derive(Resource)]
struct NumPipesToSpawn(u32);

impl FromWorld for NumPipesToSpawn {
    fn from_world(world: &mut World) -> Self {
        let window = world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(world);

        let num_pipes = (window.width() / 400.0) as u32;

        NumPipesToSpawn(num_pipes + 1)
    }
}

fn setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    assets: Res<MyAssets>,
    num_pipes: Res<NumPipesToSpawn>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2d);

    let sprite_size = 100.0;

    commands.spawn({
        let mut sprite = Sprite::from_image(assets.background.clone());
        sprite.custom_size = Some(Vec2::new(1920.0, 1080.0));
        sprite
    });

    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(500, 500), 12, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let mut sprite = Sprite::from_atlas_image(assets.corgi.clone(), texture_atlas_handle.into());
    sprite.flip_x = true;
    sprite.custom_size = Some(Vec2::new(sprite_size, sprite_size));
    commands
        .spawn((sprite, Transform::from_xyz(-window.width() / 4.0, 0.0, 1.0)))
        .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Collider::cuboid(sprite_size / 2.0, sprite_size / 2.0))
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
    let ground_size = Vec2::new(window.width(), window.height() / 10.0);
    commands
        .spawn((
            Sprite::from_color(
                Color::srgb(0.14, 0.75, 0.46),
                Vec2::new(ground_size.x, ground_size.y),
            ),
            Transform::from_xyz(0.0, -window.height() / 2.0 + ground_size.y / 2.0, 1.0),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(ground_size.x / 2.0, ground_size.y / 2.0))
        .insert(Ground);

    for index in 0..(num_pipes.0) {
        commands.queue(SpawnPipe {
            image: assets.hill.clone(),
            transform: Transform::from_xyz(200.0 + 400.0 * index as f32, 0.0, 1.0),
        });
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
                continue;
            };
            // sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
            texture_atlas.index = (texture_atlas.index + 1) % 12;
        }
    }
}

fn corgi_control(
    mut corgi: Query<(&mut Velocity, &mut ExternalImpulse), With<Corgi>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        let (mut velocity, mut impulse) = corgi.single_mut();
        impulse.impulse = Vect::new(0.0, 200.0);
        velocity.linvel = Vec2::new(0.0, 0.0);
    }
}

#[allow(clippy::type_complexity)]
fn align_to_window(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut corgis: Query<&mut Transform, With<Corgi>>,
    mut ground: Query<(&mut Sprite, &mut Transform, &mut Collider), (With<Ground>, Without<Corgi>)>,
) {
    let window = window_query.get_single().unwrap();
    for mut corgi in corgis.iter_mut() {
        corgi.translation.x = -window.width() / 4.0;
    }

    for (mut sprite, mut transform, mut collider) in ground.iter_mut() {
        let ground_size = Vec2::new(window.width(), window.height() / 10.0);
        sprite.custom_size = Some(Vec2::new(ground_size.x, ground_size.y));

        transform.translation.y = -window.height() / 2.0 + ground_size.y / 2.0;
        *collider = Collider::cuboid(ground_size.x / 2.0, ground_size.y / 2.0)
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    gates: Query<Entity, With<PointsGate>>,
    corgi: Query<Entity, With<Corgi>>,
    mut score: ResMut<Score>,
) {
    for collision_event in collision_events.read() {
        let corgi = corgi.single();
        match collision_event {
            CollisionEvent::Started(a, b, _flags) => {
                if let Some((_corgi, other)) = if corgi == *a {
                    Some((a, b))
                } else if corgi == *b {
                    Some((b, a))
                } else {
                    None
                } {
                    if let Ok(_gate_entity) = gates.get(*other) {
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
    window_query: Query<&Window, With<PrimaryWindow>>,
    assets: Res<MyAssets>,
) {
    let window = window_query.get_single().unwrap();
    for (entity, transform) in pipes.iter() {
        if transform.translation.x < -window.width() / 2.0 {
            commands.entity(entity).despawn_recursive();
            commands.queue(SpawnPipe {
                image: assets.hill.clone(),
                transform: Transform::from_xyz(window.width() - 200.0, 0.0, 1.0),
            });
        }
    }
}
