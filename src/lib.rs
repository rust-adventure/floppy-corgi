use std::time::Duration;

use bevy::{image::ImageLoaderSettings, prelude::*, time::common_conditions::on_timer};

pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
pub const PLAYER_SIZE: f32 = 25.0;
const PIPE_SIZE: Vec2 = Vec2::new(32., CANVAS_SIZE.y);
const GAP_SIZE: f32 = 100.0;
pub const PIPE_SPEED: f32 = 200.0;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                despawn_pipes,
                shift_pipes_to_the_left,
                spawn_pipes.run_if(on_timer(Duration::from_millis(1000))),
            ),
        );
    }
}

#[derive(Component, Default, Clone)]
pub struct Pipe;

#[derive(Component, Default, Clone)]
pub struct PipeTop;

#[derive(Component, Default, Clone)]
pub struct PipeBottom;

#[derive(Component, Default, Clone)]
pub struct PointsGate;

fn spawn_pipes(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>) {
    let image = asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings
                .sampler
                .get_or_init_descriptor()
                .set_filter(bevy::image::ImageFilterMode::Nearest);
        })
        .load("pipe.png");

    let image_mode = SpriteImageMode::Sliced(TextureSlicer {
        border: BorderRect::axes(8., 19.),
        center_scale_mode: SliceScaleMode::Stretch,
        ..default()
    });

    let transform = Transform::from_xyz(CANVAS_SIZE.x / 2., 0.0, 1.0);
    let gap_y_position = (time.elapsed_secs() * 4.2309875).sin() * CANVAS_SIZE.y / 4.;
    let pipe_offset = PIPE_SIZE.y / 2.0 + GAP_SIZE / 2.0;

    commands.spawn_scene(bsn! {
        #Pipe
        Pipe
        template_value(transform)
        Visibility::Visible
        Children[
            #PipeTop
            PipeTop
            Sprite {
                image: {image.clone()},
                custom_size: PIPE_SIZE,
                image_mode: {image_mode.clone()},
            }
            Transform::from_xyz(
                0.0,
                pipe_offset + gap_y_position,
                1.0,
            ),

            #PointsGate
            PointsGate
            Visibility::Hidden
            Sprite {
                color: Color::WHITE,
                custom_size: Vec2::new(
                    10.0, GAP_SIZE,
                ),
            }
            Transform::from_xyz(
                0.0,
                gap_y_position,
                1.0,
            ),

            #PipeBottom
            PipeBottom
            Sprite {
                image,
                custom_size: PIPE_SIZE,
                image_mode,
            }
            Transform::from_xyz(
                0.0,
                -pipe_offset + gap_y_position,
                1.0,
            ),
        ]
    });
}

pub fn shift_pipes_to_the_left(mut pipes: Query<&mut Transform, With<Pipe>>, time: Res<Time>) {
    for mut pipe in &mut pipes {
        pipe.translation.x -= PIPE_SPEED * time.delta_secs();
    }
}

fn despawn_pipes(mut commands: Commands, pipes: Query<(Entity, &Transform), With<Pipe>>) {
    for (entity, transform) in pipes.iter() {
        if transform.translation.x < -(CANVAS_SIZE.x / 2.0 + PIPE_SIZE.x) {
            commands.entity(entity).despawn();
        }
    }
}
