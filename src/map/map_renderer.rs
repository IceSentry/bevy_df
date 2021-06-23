use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashSet;

use crate::map::map_generator::TileType;

use super::{map_generator::MapGeneratorData, MapGeneratedEvent, HEIGHT, WIDTH, Z_LEVELS};

pub struct MapRendererData {
    pub visible_layers: Vec<bool>,
    pub current_z_level: u16,
}

pub fn update_map_state(
    mut map_renderer_data: ResMut<MapRendererData>,
    mut map_query: MapQuery,
    mut tile_query: Query<&mut Tile>,
) {
    if !map_renderer_data.is_changed() {
        return;
    }
    for layer_id in 0..Z_LEVELS {
        if layer_id > map_renderer_data.current_z_level
            && map_renderer_data.visible_layers[layer_id as usize]
        {
            set_layer_visibility(
                &mut map_renderer_data,
                &mut map_query,
                &mut tile_query,
                layer_id,
                false,
            );
        } else if layer_id <= map_renderer_data.current_z_level
            && !map_renderer_data.visible_layers[layer_id as usize]
        {
            set_layer_visibility(
                &mut map_renderer_data,
                &mut map_query,
                &mut tile_query,
                layer_id,
                true,
            );
        }
    }
}

fn set_layer_visibility(
    map_renderer_data: &mut ResMut<MapRendererData>,
    map_query: &mut MapQuery,
    tile_query: &mut Query<&mut Tile>,
    layer_id: u16,
    visibility: bool,
) {
    iterate_layer(|x, y| {
        let position = UVec2::new(x as u32, y as u32);
        if let Ok(tile_entity) = map_query.get_tile_entity(position, 0u16, layer_id) {
            if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                tile.visible = visibility;
            }
        }
        map_query.notify_chunk_for_tile(position, 0u16, layer_id);
    });
    map_renderer_data.visible_layers[layer_id as usize] = visibility;
}

fn iterate_layer<F: FnMut(u32, u32)>(mut f: F) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            f(x as u32, y as u32);
        }
    }
}

pub fn draw_map(
    mut tile_query: Query<(&mut Tile, &TileParent, &UVec2)>,
    mut map_query: MapQuery,
    map_generator_data: Res<MapGeneratorData>,
    mut events: EventReader<MapGeneratedEvent>,
) {
    for _ in events.iter() {
        let mut chunks = HashSet::new();
        for (mut tile, tile_parent, pos) in tile_query.iter_mut() {
            let layer = &map_generator_data.layers[tile_parent.layer_id as usize];
            let tile_data = layer.get_tile(pos.x as usize, pos.y as usize);
            tile.texture_index = if tile_data.visible {
                match tile_data.value {
                    TileType::Air => 12,
                    TileType::Water => 1,
                    TileType::Grass => 4,
                    TileType::Rock => 5,
                }
            } else {
                15 // black
            };

            // TODO use this to figure out which corner piece to use
            // let neighbors = map_query.get_tile_neighbors(*pos, 0u16, tile_parent.layer_id);

            chunks.insert(tile_parent.chunk);
        }

        for chunk_entity in chunks.drain() {
            map_query.notify_chunk(chunk_entity);
        }
    }
}
