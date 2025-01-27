use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use noise::{BasicMulti, NoiseFn, Perlin};

const NOISE_SCALE: f64 = 0.0234;

#[derive(Component)]
pub struct Pipe {
    #[allow(dead_code)]
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
    fn apply(self, world: &mut World) {
        let window = world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(world);
        let time = world.get_resource::<Time>().unwrap();
        // Pipe
        let pipe_size = Vec2::new(200.0, window.height());
        let noise = BasicMulti::<Perlin>::default();
        let center_of_opening = noise.get([
            self.transform.translation.x as f64 / NOISE_SCALE,
            time.elapsed_secs_f64(),
        ]);
        let position = (window.height() as f64 / 2.0) * center_of_opening;

        let gap_size = 200.0;

        world
            .spawn((self.transform, Visibility::Visible))
            .insert(RigidBody::KinematicVelocityBased)
            .insert(Velocity::linear(Vec2::new(-100.0, 0.0)))
            .with_children(|builder| {
                // top pipe
                let mut sprite =
                    Sprite::from_color(Color::srgb(46. / 255., 139. / 255., 87. / 255.), pipe_size);
                sprite.flip_y = true;
                sprite.image = self.image.clone();
                builder
                    .spawn((
                        sprite,
                        Transform::from_xyz(
                            0.0,
                            (pipe_size.y / 2.0 + gap_size / 2.0) + position as f32,
                            1.0,
                        ),
                    ))
                    .insert(Collider::capsule(
                        Vec2::new(0.0, -200.0),
                        Vec2::new(0.0, pipe_size.y),
                        100.0,
                    ))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(PipeTop);

                // Gap Sensor
                builder
                    .spawn((
                        Sprite::from_color(Color::NONE, Vec2::new(10.0, gap_size)),
                        Transform::from_xyz(0.0, position as f32, 1.0),
                    ))
                    .insert(Sensor)
                    .insert(Collider::cuboid(5.0, gap_size / 2.0))
                    .insert(PointsGate)
                    .insert(ActiveEvents::COLLISION_EVENTS);

                // bottom pipe
                let mut sprite =
                    Sprite::from_color(Color::srgb(46. / 255., 139. / 255., 87. / 255.), pipe_size);
                sprite.image = self.image;

                builder
                    .spawn((
                        sprite,
                        Transform::from_xyz(
                            0.0,
                            -pipe_size.y / 2.0 - gap_size / 2.0 + position as f32,
                            1.0,
                        ),
                    ))
                    .insert(Collider::capsule(
                        Vec2::new(0.0, 200.0),
                        Vec2::new(0.0, -pipe_size.y),
                        100.0,
                    ))
                    .insert(PipeBottom);
            })
            .insert(Pipe {
                noise: center_of_opening,
            });
    }
}
