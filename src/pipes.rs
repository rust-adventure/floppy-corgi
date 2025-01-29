use bevy::{color::palettes::css::SEA_GREEN, prelude::*};
use bevy_transform_interpolation::prelude::TransformInterpolation;
use noise::{BasicMulti, NoiseFn, Perlin};

use crate::CANVAS_SIZE;

#[derive(Component)]
pub struct Pipe {
    noise: f64,
}

#[derive(Component)]
pub struct PipeTop;

#[derive(Component)]
pub struct PipeBottom;

#[derive(Component)]
pub struct PointsGate;

#[derive(Component)]
pub struct Scored;

const GAP_SIZE: f32 = 125.0;

#[derive(Event)]
pub struct ScorePoint;

#[derive(Event)]
pub struct EndGame;

pub struct SpawnPipe {
    pub transform: Transform,
    pub image: Handle<Image>,
}

impl Command for SpawnPipe {
    fn apply(self, world: &mut World) {
        let time = world.get_resource::<Time>().unwrap();

        // Pipe
        let pipe_size = Vec2::new(48.0, 146.0);
        let noise: BasicMulti<Perlin> = BasicMulti::new(0);
        let center_of_opening = noise.get([
            self.transform.translation.x as f64 / 0.0234,
            time.elapsed_secs_f64(),
        ]);
        let position = (CANVAS_SIZE.y as f64 / 2.0)
            * center_of_opening;

        world
            .spawn((
                self.transform,
                Visibility::Visible,
                Pipe {
                    noise: center_of_opening,
                },
                TransformInterpolation,
            ))
            .with_children(|builder| {
                // top pipe
                builder
                    .spawn((
                        Sprite {
                            color: SEA_GREEN.into(),
                            custom_size: Some(Vec2::new(
                                pipe_size.x,
                                pipe_size.y,
                            )),
                            flip_y: true,
                            image: self.image.clone(),
                            ..default()
                        },
                        Transform::from_xyz(
                            0.0,
                            (pipe_size.y / 2.0
                                + GAP_SIZE / 2.0)
                                + position as f32,
                            1.0,
                        ),
                    ))
                    // .insert(Collider::capsule(
                    //     Vec2::new(0.0, -200.0),
                    //     Vec2::new(0.0, pipe_size.y),
                    //     100.0,
                    // ))
                    // // .insert(Collider::cuboid(
                    // //     pipe_size.x / 2.0,
                    // //     pipe_size.y / 2.0,
                    // // ))
                    // .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(PipeTop);

                // Gap Sensor
                builder.spawn((
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
                    // Sensor,
                    PointsGate,
                ));
                // .insert(Collider::cuboid(
                //     5.0,
                //     GAP_SIZE / 2.0,
                // ))
                // .insert(ActiveEvents::COLLISION_EVENTS);

                // bottom pipe
                builder.spawn((
                    Sprite {
                        color: SEA_GREEN.into(),
                        custom_size: Some(Vec2::new(
                            pipe_size.x,
                            pipe_size.y,
                        )),
                        image: self.image,
                        ..default()
                    },
                    Transform::from_xyz(
                        0.0,
                        -pipe_size.y / 2.0 - GAP_SIZE / 2.0
                            + position as f32,
                        1.0,
                    ),
                    PipeBottom,
                ));
                // .insert(Collider::capsule(
                //     Vec2::new(0.0, 200.0),
                //     Vec2::new(0.0, -pipe_size.y),
                //     100.0,
                // ))
                // .insert(Collider::cuboid(
                //     pipe_size.x / 2.0,
                //     pipe_size.y / 2.0,
                // ))
                // .insert(PipeBottom);
            });
    }
}
