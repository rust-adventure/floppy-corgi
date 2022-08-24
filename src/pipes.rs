use bevy::{ecs::system::Command, prelude::*};
use bevy_rapier2d::prelude::*;
use noise::{BasicMulti, NoiseFn};

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

pub struct SpawnPipe {
    pub transform: Transform,
    pub image: Handle<Image>,
}

impl Command for SpawnPipe {
    fn write(self, world: &mut World) {
        let time = world
            .get_resource::<Time>()
            .unwrap()
            .time_since_startup();
        let window = world
            .get_resource::<Windows>()
            .unwrap()
            .primary();
        // Pipe
        let pipe_size = Vec2::new(200.0, window.height());
        let noise = BasicMulti::new();
        let center_of_opening = noise.get([
            self.transform.translation.x as f64 / 0.0234,
            time.as_secs_f64(),
        ]);
        let position = ((window.height() as f64 / 2.0)
            * center_of_opening);

        let gap_size = 200.0;

        world
            .spawn()
            .insert_bundle(SpatialBundle {
                transform: self.transform,
                visibility: Visibility { is_visible: true },
                ..default()
            })
            .insert(RigidBody::KinematicVelocityBased)
            .insert(Velocity::linear(Vec2::new(
                -100.0, 0.0,
            )))
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
                            flip_y: true,
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            0.0,
                            (pipe_size.y / 2.0
                                + gap_size / 2.0)
                                + position as f32,
                            1.0,
                        ),
                        texture: self.image.clone(),
                        ..default()
                    })
                    .insert(Collider::capsule(
                        Vec2::new(0.0, -200.0),
                        Vec2::new(0.0, pipe_size.y),
                        100.0,
                    ))
                    // .insert(Collider::cuboid(
                    //     pipe_size.x / 2.0,
                    //     pipe_size.y / 2.0,
                    // ))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(PipeTop);

                // Gap Sensor
                builder
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::NONE,
                            custom_size: Some(Vec2::new(
                                10.0, gap_size,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            0.0,
                            position as f32,
                            1.0,
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
                            -pipe_size.y / 2.0
                                - gap_size / 2.0
                                + position as f32,
                            1.0,
                        ),
                        texture: self.image,
                        ..default()
                    })
                    .insert(Collider::capsule(
                        Vec2::new(0.0, 200.0),
                        Vec2::new(0.0, -pipe_size.y),
                        100.0,
                    ))
                    // .insert(Collider::cuboid(
                    //     pipe_size.x / 2.0,
                    //     pipe_size.y / 2.0,
                    // ))
                    .insert(PipeBottom);
            })
            .insert(Pipe {
                noise: center_of_opening,
            });
    }
}
