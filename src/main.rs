use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    sprite::{Anchor, Material2dPlugin},
    time::common_conditions::on_timer,
    window::PrimaryWindow,
};
use bevy_asset_loader::prelude::*;
use bevy_transform_interpolation::prelude::{
    TransformInterpolation, TransformInterpolationPlugin,
};
use floppy_corgi::{
    CANVAS_SIZE,
    background_material::BackgroundMaterial,
    pipes::{
        EndGame, Pipe, PipeBottom, PipeTop, PointsGate,
        ScorePoint, Scored, SpawnPipe,
    },
};

fn main() {
    App::new()
        .init_resource::<Score>()
        // .insert_resource(ImageSettings::default_nearest())
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Floppy Corgi".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            TransformInterpolationPlugin::default(),
            Material2dPlugin::<BackgroundMaterial>::default(
            ),
        ))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), setup)
        .init_state::<MyStates>()
        .add_systems(
            Update,
            (
                animate_sprite,
                corgi_control,
                despawn_pipes,
                check_collisions,
                spawn_pipes.run_if(on_timer(
                    Duration::from_millis(1000),
                )),
            )
                .run_if(in_state(MyStates::Next)),
        )
        .add_systems(
            FixedUpdate,
            (gravity, pipes_to_the_left)
                .run_if(in_state(MyStates::Next)),
        )
        .add_observer(|_trigger: Trigger<EndGame>| {
            dbg!("trigger endgame");
        })
        .add_observer(|_trigger: Trigger<ScorePoint>| {
            dbg!("trigger point");
        })
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(
        tile_size_x = 500,
        tile_size_y = 500,
        columns = 12,
        rows = 1
    ))]
    corgi_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "corgi.png")]
    corgi: Handle<Image>,

    #[asset(path = "hill_large.png")]
    #[asset(image(sampler(filter = nearest)))]
    hill: Handle<Image>,

    #[asset(path = "background_color_grass.png")]
    #[asset(image(sampler(filter = nearest, wrap = repeat)))]
    background: Handle<Image>,
}

#[derive(Component)]
struct Corgi;

#[derive(Component)]
struct Ground;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Component)]
struct Gravity;

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Component, Default)]
struct Acceleration(f32);

fn setup(
    mut commands: Commands,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: CANVAS_SIZE.x,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.x,
        ))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: assets.background.clone(),
        })),
    ));

    commands.spawn((
        Sprite {
            flip_x: true,
            custom_size: Some(Vec2::splat(25.0)),
            image: assets.corgi.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.corgi_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
        AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )),
        TransformInterpolation,
        Corgi,
        Gravity,
        Velocity(0.),
        Acceleration(10.),
    ));

    // Sky
    commands.spawn((
        Sprite {
            color: Color::srgb(0.81, 0.94, 0.99),
            custom_size: Some(Vec2::new(
                CANVAS_SIZE.x,
                CANVAS_SIZE.y * 4.,
            )),
            anchor: Anchor::BottomCenter,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        Ground,
    ));
    // Ground
    commands.spawn((
        Sprite {
            color: Color::srgb(0.14, 0.75, 0.46),
            custom_size: Some(Vec2::new(
                CANVAS_SIZE.x,
                CANVAS_SIZE.y * 4.,
            )),
            anchor: Anchor::TopCenter,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        Ground,
    ));
}

#[derive(
    Default, Clone, Eq, PartialEq, Debug, Hash, States,
)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let atlas =
                sprite.texture_atlas.as_mut().unwrap();
            let texture_count = layouts
                .get(&atlas.layout)
                .unwrap()
                .textures
                .len();
            atlas.index = (atlas.index + 1) % texture_count;
        }
    }
}

fn gravity(
    mut transforms: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Acceleration,
        ),
        With<Gravity>,
    >,
    time: Res<Time>,
) {
    let gravity: f32 = -2000.;

    for (mut transform, mut velocity, mut acceleration) in
        &mut transforms
    {
        acceleration.0 += gravity * time.delta_secs();

        velocity.0 += acceleration.0 * time.delta_secs();

        transform.translation.y +=
            velocity.0 * time.delta_secs();
    }
}

fn corgi_control(
    mut corgi: Single<
        (&mut Velocity, &mut Acceleration),
        With<Corgi>,
    >,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        corgi.0.0 = 200.;
        corgi.1.0 = 0.;
    }
}

fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for (entity, transform) in pipes.iter() {
        if transform.translation.x < -window.width() / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_pipes(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    assets: Res<MyAssets>,
) {
    commands.queue(SpawnPipe {
        image: assets.hill.clone(),
        transform: Transform::from_xyz(
            window.width() - 200.0,
            0.0,
            1.0,
        ),
    });
}

const PIPE_SPEED: f32 = 200.;

pub fn pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Pipe>>,
    time: Res<Time>,
) {
    for mut pipe in &mut pipes {
        pipe.translation.x -=
            PIPE_SPEED * time.delta_secs();
    }
}

fn check_collisions(
    mut commands: Commands,
    corgi: Single<(&Sprite, &GlobalTransform), With<Corgi>>,
    pipe_segments: Query<
        (&GlobalTransform, &Sprite),
        Or<(With<PipeTop>, With<PipeBottom>)>,
    >,
    pipe_gaps: Query<
        (&GlobalTransform, &Sprite, Entity),
        (With<PointsGate>, Without<Scored>),
    >,
    mut gizmos: Gizmos,
) {
    let corgi_collider = Aabb2d::new(
        corgi.1.translation().truncate(),
        corgi.1.scale().truncate() / 2.,
    );
    gizmos.rect_2d(
        corgi.1.translation().truncate(),
        corgi.0.custom_size.unwrap().xy(),
        Color::BLACK,
    );

    for (pipe_transform, sprite) in &pipe_segments {
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().truncate(),
            sprite.custom_size.unwrap().xy() / 2.,
        );
        gizmos.rect_2d(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap().xy(),
            Color::BLACK,
        );
        if corgi_collider.intersects(&pipe_collider) {
            commands.trigger(EndGame);
        }
    }

    for (pipe_transform, sprite, entity) in &pipe_gaps {
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().truncate(),
            sprite.custom_size.unwrap().xy() / 2.,
        );
        gizmos.rect_2d(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap().xy(),
            Color::BLACK,
        );
        if corgi_collider.intersects(&pipe_collider) {
            commands.trigger(ScorePoint);
            commands.entity(entity).insert(Scored);
        }
    }
}
// BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
// Aabb2d::new(
//     collider_transform.translation.truncate(),
//     collider_transform.scale.truncate() / 2.,
// ),
