use bevy::{
    color::palettes::tailwind::*,
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    render::camera::ScalingMode,
    sprite::Material2dPlugin,
};
use bevy_asset_loader::prelude::*;
use bevy_transform_interpolation::prelude::{
    TransformInterpolation, TransformInterpolationPlugin,
};
use floppy_corgi::{
    CANVAS_SIZE, CORGI_SIZE, MyAssets,
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
            TransformInterpolationPlugin::default(),
            Material2dPlugin::<BackgroundMaterial>::default(
            ),
            PipePlugin,
        ))
        .add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(AppState::Next), setup)
        .init_state::<AppState>()
        .add_systems(
            Update,
            (
                animate_sprite,
                corgi_control,
                score_update,
            )
                .run_if(in_state(AppState::Next)),
        )
        .add_systems(
            FixedUpdate,
            (
                gravity,
                check_collisions,
                check_in_bounds,
            )
                .run_if(in_state(AppState::Next)),
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
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
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
            color_texture: assets.background.clone(),
        })),
    ));

    commands.spawn((
        Sprite {
            flip_x: true,
            custom_size: Some(Vec2::splat(CORGI_SIZE)),
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
    ));

    commands
        .spawn((
            Text::default(),
            TextLayout::new_with_justify(
                JustifyText::Center,
            ),
            Node {
                width: Val::Percent(100.),
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            TextColor(SLATE_950.into()),
        ))
        .with_child((
            TextSpan::new("0"),
            (
                TextFont {
                    font_size: 33.0,
                    // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                    ..default()
                },
                TextColor(SLATE_950.into()),
            ),
            ScoreText,
        ));
}

#[derive(
    Default, Clone, Eq, PartialEq, Debug, Hash, States,
)]
enum AppState {
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
        dbg!(velocity.0);

        transform.translation.y +=
            velocity.0 * time.delta_secs();
    }
}

fn corgi_control(
    mut corgi: Single<&mut Velocity, With<Corgi>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
    ]) {
        corgi.0 = 400.;
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
        corgi.1.translation().truncate(),
        corgi.1.scale().truncate() / 2.,
    );

    #[cfg(feature = "debug-colliders")]
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

        #[cfg(feature = "debug-colliders")]
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
