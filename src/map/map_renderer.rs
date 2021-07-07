use bevy::{prelude::*, tasks::ComputeTaskPool, utils::Instant};
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashSet;

use super::{
    map_generator::{MapGeneratorData, TileType},
    MapGeneratedEvent, HEIGHT, WIDTH, Z_LEVELS,
};

// TODO
// * add darker shade to hidden tiles
// * use this to figure out which corner piece to use
//      let neighbors = map_query.get_tile_neighbors(*pos, 0u16, tile_parent.layer_id);
// * merge MapRendererData and MapGeneratorData??

pub struct MapRendererData {
    pub visible_layers: Vec<bool>,
    pub current_z_level: u16,
}

pub fn update_layer_visibility(
    mut tile_query: Query<(&mut Tile, &TileParent)>,
    mut chunk_query: Query<&mut Chunk>,
    mut map_renderer_data: ResMut<MapRendererData>,
    pool: Res<ComputeTaskPool>,
) {
    if !map_renderer_data.is_changed() {
        return;
    }
    info!("updating layer visibility...");
    let start = Instant::now();

    tile_query.par_for_each_mut(&pool, 128, |(mut tile, tile_parent)| {
        let z_level = tile_parent.layer_id;
        let is_layer_visible = map_renderer_data.visible_layers[z_level as usize];
        if z_level > map_renderer_data.current_z_level && is_layer_visible {
            tile.visible = false;
        } else if z_level <= map_renderer_data.current_z_level && !is_layer_visible {
            tile.visible = true;
        }
    });

    for mut chunk_entity in chunk_query.iter_mut() {
        chunk_entity.needs_remesh = true;
    }

    for z_level in 0..Z_LEVELS {
        let is_layer_visible = map_renderer_data.visible_layers[z_level as usize];
        if z_level > map_renderer_data.current_z_level && is_layer_visible {
            map_renderer_data.visible_layers[z_level as usize] = false;
        } else if z_level <= map_renderer_data.current_z_level && !is_layer_visible {
            map_renderer_data.visible_layers[z_level as usize] = true;
        }
    }

    info!(
        "updating layer visibility...done elapsed: {:?}",
        start.elapsed()
    );
}

pub fn set_map_textures(
    mut tile_query: Query<(&mut Tile, &TileParent, &UVec2)>,
    mut chunk_query: Query<&mut Chunk>,
    pool: Res<ComputeTaskPool>,
    map_generator_data: Res<MapGeneratorData>,
    mut events: EventReader<MapGeneratedEvent>,
) {
    if events.iter().count() == 0 {
        return;
    }
    info!("setting map textures...");
    let start = Instant::now();
    tile_query.par_for_each_mut(&pool, 128, |(mut tile, tile_parent, pos)| {
        let layer = &map_generator_data.layers[tile_parent.layer_id as usize];
        let tile_data = layer.get_tile(pos.x as usize, pos.y as usize);

        tile.texture_index = match tile_data.value {
            TileType::Air => 1,
            TileType::Water => 2,
            TileType::Grass => 3,
            TileType::Dirt => 4,
            TileType::Rock => 5,
        };
    });

    for mut chunk_entity in chunk_query.iter_mut() {
        chunk_entity.needs_remesh = true;
    }
    info!("setting map textures...done elapsed: {:?}", start.elapsed());
}
