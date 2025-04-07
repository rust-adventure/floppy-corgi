use std::time::Duration;

use bevy::{
    color::palettes::css::SEA_GREEN, prelude::*,
    time::common_conditions::on_timer,
};
use bevy_transform_interpolation::prelude::TransformInterpolation;
use noise::{BasicMulti, NoiseFn, Perlin};

use crate::{CANVAS_SIZE, MyAssets};

const PIPE_SPEED: f32 = 200.;
const GAP_SIZE: f32 = 125.0;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                shift_pipes_to_the_left,
                despawn_pipes,
                spawn_pipes.run_if(on_timer(
                    Duration::from_millis(1000),
                )),
            ),
        );
    }
}

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct PipeTop;

#[derive(Component)]
pub struct PipeBottom;

#[derive(Component)]
pub struct PointsGate;

fn spawn_pipes(
    mut commands: Commands,
    time: Res<Time>,
    my_assets: Res<MyAssets>,
) {
    let transform =
        Transform::from_xyz(CANVAS_SIZE.x + 10.0, 0.0, 1.0);
    let hill = my_assets.hill.clone();

    // Pipe
    let pipe_size = Vec2::new(48.0, 146.0);
    let noise: BasicMulti<Perlin> = BasicMulti::new(0);
    let center_of_opening = noise.get([
        transform.translation.x as f64 / 0.0234,
        time.elapsed_secs_f64(),
    ]);
    let position =
        (CANVAS_SIZE.y as f64 / 2.0) * center_of_opening;

    commands.spawn((
        transform,
        Visibility::Visible,
        Pipe,
        TransformInterpolation,
        children![
            (
                Sprite {
                    color: SEA_GREEN.into(),
                    custom_size: Some(Vec2::new(
                        pipe_size.x,
                        pipe_size.y,
                    )),
                    flip_y: true,
                    image: hill.clone(),
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    (pipe_size.y / 2.0 + GAP_SIZE / 2.0)
                        + position as f32,
                    1.0,
                ),
                PipeTop
            ),
            (
                Sprite {
                    color: Color::NONE,
                    custom_size: Some(Vec2::new(
                        10.0, GAP_SIZE,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    position as f32,
                    1.0,
                ),
                PointsGate,
            ),
            (
                Sprite {
                    color: SEA_GREEN.into(),
                    custom_size: Some(Vec2::new(
                        pipe_size.x,
                        pipe_size.y,
                    )),
                    image: hill,
                    ..default()
                },
                Transform::from_xyz(
                    0.0,
                    -pipe_size.y / 2.0 - GAP_SIZE / 2.0
                        + position as f32,
                    1.0,
                ),
                PipeBottom,
            )
        ],
    ));
}

fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>,
) {
    let pipe_width = 48.;
    for (entity, transform) in pipes.iter() {
        if transform.translation.x
            < -(CANVAS_SIZE.x / 2.0 + pipe_width)
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn shift_pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Pipe>>,
    time: Res<Time>,
) {
    for mut pipe in &mut pipes {
        pipe.translation.x -=
            PIPE_SPEED * time.delta_secs();
    }
}
