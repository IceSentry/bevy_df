use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, SuperSimplex};

use crate::utils::inverse_lerp;

use super::{MapGeneratedEvent, HEIGHT, WIDTH};

#[derive(Inspectable)]
pub struct NoiseSettings {
    #[inspectable(visual, min = Vec2::splat(-2.0), max = Vec2::splat(2.0))]
    pub offset: Vec2,
    #[inspectable(min = 1, max = 8)]
    pub octaves: i32,
    #[inspectable(min = 0.0, max = 4.0, speed = 0.1)]
    pub lacunarity: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.1)]
    pub persistence: f32,
    #[inspectable(min = 0.1, max = 2.0, speed = 0.1)]
    pub scale: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            offset: Vec2::splat(0.0),
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
            scale: 1.0,
        }
    }
}

pub struct MapGeneratorData {
    pub elevation: Vec<f32>,
}

pub fn generate_map(
    noise: Res<SuperSimplex>,
    noise_settings: Res<NoiseSettings>,
    mut map: ResMut<MapGeneratorData>,
    mut event: EventWriter<MapGeneratedEvent>,
) {
    if !noise_settings.is_changed() {
        return;
    }

    let bounds = (-1.0, 1.0);
    let extent = bounds.1 - bounds.0;
    let step = extent as f64 / WIDTH as f64;

    let mut min = std::f32::MAX;
    let mut max = std::f32::MIN;

    for y in 0..HEIGHT {
        let current_y = bounds.0 + step * y as f64;
        for x in 0..WIDTH {
            let current_x = bounds.0 + step * x as f64;

            let mut amplitude = 1.;
            let mut frequency = 1.;
            let mut elevation = 0.0;

            for _ in 0..noise_settings.octaves {
                let mut sample_point = Vec2::new(current_x as f32, current_y as f32);
                sample_point = sample_point / noise_settings.scale * frequency;
                sample_point += noise_settings.offset;

                let noise_value = noise.get([sample_point.x as f64, sample_point.y as f64, 0.0]);

                elevation += noise_value as f32 * amplitude;

                amplitude *= noise_settings.persistence;
                frequency *= noise_settings.lacunarity;
            }
            if elevation > max {
                max = elevation;
            } else if elevation < min {
                min = elevation;
            }

            map.elevation[y * WIDTH + x] = elevation;
        }
    }
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let map_idx = y * WIDTH + x;
            // TODO maybe do this in the renderer since it's already looping on everything
            map.elevation[map_idx] = inverse_lerp(min, max, map.elevation[map_idx]);
        }
    }

    event.send(MapGeneratedEvent);
}
