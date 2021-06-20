use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, SuperSimplex};

use super::{MapGeneratedEvent, HEIGHT, WIDTH};

#[derive(Inspectable)]
pub struct NoiseSettings {
    #[inspectable(visual, min = Vec2::splat(-2.0), max = Vec2::splat(2.0))]
    pub offset: Vec2,
    #[inspectable(min = 1, max = 8)]
    pub octaves: i32,
    #[inspectable(min = 1.0, max = 10.0)]
    pub base_frequency: f32,
    #[inspectable(min = 0.0, max = 10.0)]
    pub roughness: f32,
    #[inspectable(min = 0.05, max = 2.5, speed = 0.05)]
    pub persistence: f32,
    #[inspectable(min = 0.0, max = 4.0)]
    pub min_value: f32,
    #[inspectable(min = 0.05, max = 2.5, speed = 0.05)]
    pub strength: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            offset: Vec2::splat(0.0),
            octaves: 8,
            base_frequency: 1.0,
            roughness: 5.0,
            persistence: 0.05,
            min_value: 0.0,
            strength: 0.05,
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

    for y in 0..HEIGHT {
        let current_y = bounds.0 + step * y as f64;
        for x in 0..WIDTH {
            let current_x = bounds.0 + step * x as f64;

            let mut elevation = 0.0;
            let mut frequency = noise_settings.base_frequency;
            let mut amplitude = 1.0;
            let mut amplitude_sum = 0.0;

            for _ in 0..noise_settings.octaves {
                let point = Vec2::new(current_x as f32, current_y as f32) * frequency
                    + noise_settings.offset;

                let noise_value = noise.get([point.x as f64, point.y as f64, 0.0]);
                let noise_value = (noise_value + 1.0) * 0.5;
                elevation += noise_value as f32 * amplitude;
                frequency *= noise_settings.roughness;
                amplitude *= noise_settings.persistence;
                amplitude_sum += amplitude
            }
            elevation /= amplitude_sum;
            elevation = (elevation - noise_settings.min_value).max(0.0);
            let map_idx = y * WIDTH + x;
            map.elevation[map_idx] = elevation * noise_settings.strength;
        }
    }
    event.send(MapGeneratedEvent);
}
