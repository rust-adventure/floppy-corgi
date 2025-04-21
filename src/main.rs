use bevy::{
    color::palettes::tailwind::*,
    image::{
        ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    sprite::Material2dPlugin,
};
use bevy_transform_interpolation::prelude::TransformInterpolationPlugin;
use floppy_corgi::{
    CANVAS_SIZE, CORGI_SIZE,
    background_material::BackgroundMaterial,
    pipes::{PipeBottom, PipePlugin, PipeTop, PointsGate},
};

fn main() {
    App::new()
        .init_resource::<Score>()
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
            TransformInterpolationPlugin::interpolate_all(),
            Material2dPlugin::<BackgroundMaterial>::default(
            ),
            PipePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sprite,
                corgi_control,
                score_update,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                gravity,
                check_collisions,
                check_in_bounds,
            ),
        )
        .add_observer(
            |_trigger: Trigger<EndGame>,
             mut commands: Commands,
             corgi: Single<Entity, With<Corgi>>,
             mut score: ResMut<Score>| {
                score.0 = 0;
                commands.entity(*corgi).insert((
                    Transform::from_xyz(
                        -CANVAS_SIZE.x / 4.0,
                        0.0,
                        1.0,
                    ),
                    Velocity(0.),
                ));
            },
        )
        .add_observer(
            |_trigger: Trigger<ScorePoint>,
             mut score: ResMut<Score>| {
                score.0 += 1;
            },
        )
        .run();
}

#[derive(Component)]
struct Corgi;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Component)]
struct Gravity;

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Event)]
pub struct ScorePoint;

#[derive(Event)]
pub struct EndGame;

#[derive(Component)]
struct ScoreText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut texture_atlas_layouts: ResMut<
        Assets<TextureAtlasLayout>,
    >,
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
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.x,
        ))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: asset_server.load_with_settings(
                "background_color_grass.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler =
                        ImageSampler::Descriptor(
                            ImageSamplerDescriptor {
                                address_mode_u: bevy::image::ImageAddressMode::Repeat,
                                ..default()
                            },
                        )
                },
            ),
        })),
    ));

    let texture = asset_server.load("corgi.png");
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
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
        AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )),
        Corgi,
        Gravity,
        Velocity(0.),
    ));

    commands.spawn((
        Text::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            width: Val::Percent(100.),
            padding: UiRect::all(Val::Px(10.)),
            ..default()
        },
        TextColor(SLATE_950.into()),
        children![(
            TextSpan::new("0"),
            (
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(SLATE_950.into()),
            ),
            ScoreText,
        )],
    ));
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) -> Result {
    for (mut timer, mut sprite) in &mut query {
        if timer.tick(time.delta()).just_finished() {
            let atlas =
                sprite.texture_atlas.as_mut().ok_or(
                    "Couldn't access TextureAtlas mutably",
                )?;
            let texture_count = layouts
                .get(&atlas.layout)
                .ok_or("Couldn't find TextureAtlasLayout")?
                .textures
                .len();
            atlas.index = (atlas.index + 1) % texture_count;
        }
    }
    Ok(())
}

fn gravity(
    mut transforms: Query<
        (&mut Transform, &mut Velocity),
        With<Gravity>,
    >,
    time: Res<Time>,
) {
    let gravity: f32 = -1000.;

    for (mut transform, mut velocity) in &mut transforms {
        velocity.0 += gravity * time.delta_secs();

        transform.translation.y +=
            velocity.0 * time.delta_secs();
    }
}

fn corgi_control(
    mut corgi_velocity: Single<&mut Velocity, With<Corgi>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        corgi_velocity.0 = 400.;
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
        With<PointsGate>,
    >,
    #[cfg(feature = "debug-colliders")] mut gizmos: Gizmos,
) {
    let corgi_collider = Aabb2d::new(
        corgi.1.translation().xy(),
        corgi.1.scale().xy() / 2.,
    );

    #[cfg(feature = "debug-colliders")]
    gizmos.rect_2d(
        corgi.1.translation().xy(),
        corgi.0.custom_size.unwrap(),
        Color::BLACK,
    );

    for (pipe_transform, sprite) in &pipe_segments {
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        #[cfg(feature = "debug-colliders")]
        gizmos.rect_2d(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap(),
            Color::BLACK,
        );
        if corgi_collider.intersects(&pipe_collider) {
            commands.trigger(EndGame);
        }
    }

    for (pipe_transform, sprite, entity) in &pipe_gaps {
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        #[cfg(feature = "debug-colliders")]
        gizmos.rect_2d(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap().xy(),
            Color::BLACK,
        );

        if corgi_collider.intersects(&pipe_collider) {
            commands.trigger(ScorePoint);
            commands.entity(entity).despawn();
        }
    }
}

fn score_update(
    mut query: Query<&mut TextSpan, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut span in &mut query {
        **span = format!("{:.2}", score.0);
    }
}

fn check_in_bounds(
    corgi: Single<&Transform, With<Corgi>>,
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
