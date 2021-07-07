use bevy::{prelude::*, utils::Instant};
use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, SuperSimplex};

use crate::utils::inverse_lerp;

use super::{MapGeneratedEvent, ELEVATION_MULTIPLIER, HEIGHT, WIDTH, Z_LEVELS};

// TODO
// * try to avoid using constants to make it more dynamic
// * elevation doesn't need to be in MapGeneratorData
// * generate in AsyncTaskPool

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

#[derive(Copy, Clone, Default)]
pub struct Tile {
    pub visible: bool,
    pub value: TileType,
}

#[derive(Copy, Clone, Debug)]
pub enum TileType {
    Air,
    Water,
    Grass,
    Rock,
    Dirt,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Air
    }
}

#[derive(Clone)]
pub struct Layer {
    data: Vec<Tile>,
}

impl Layer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![Tile::default(); width * height],
        }
    }
}

impl Layer {
    pub fn get_tile(&self, x: usize, y: usize) -> &Tile {
        &self.data[y * WIDTH + x]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, new_tile: Tile) {
        self.data[y * WIDTH + x] = new_tile;
    }
}

pub struct MapGeneratorData {
    pub elevation: Vec<f32>,
    pub layers: Vec<Layer>,
}

impl MapGeneratorData {
    pub fn new(width: usize, height: usize, z_levels: usize) -> Self {
        Self {
            elevation: vec![0.0; width * height],
            layers: vec![Layer::new(width, height); z_levels],
        }
    }
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
    info!("generating map...");
    let start = Instant::now();

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

    for z in 0..Z_LEVELS {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let map_idx = y * WIDTH + x;
                if z == 0 {
                    // we only need to normalize this once
                    map.elevation[map_idx] = inverse_lerp(min, max, map.elevation[map_idx]);
                }
                let elevation = map.elevation[map_idx];
                let z_level = z as f32 * ELEVATION_MULTIPLIER;
                let rounded_elevation_diff =
                    ((elevation - z_level).abs() * Z_LEVELS as f32).round() / Z_LEVELS as f32;

                let value = if rounded_elevation_diff < ELEVATION_MULTIPLIER {
                    if elevation <= 0.35 {
                        TileType::Water
                    } else {
                        TileType::Grass
                    }
                } else if z_level < elevation as f32 {
                    // everything under the grass is rocks
                    if rounded_elevation_diff <= ELEVATION_MULTIPLIER * 2.0 {
                        TileType::Dirt
                    } else {
                        TileType::Rock
                    }
                } else if z_level <= 0.35 {
                    TileType::Water
                } else {
                    TileType::Air
                };
                let tile = Tile {
                    value,
                    visible: true,
                };
                let layer = &mut map.layers[z as usize];
                layer.set_tile(x, y, tile);
            }
        }
    }
    info!("generating map...done elapsed: {:?}", start.elapsed());
    event.send(MapGeneratedEvent);
}
