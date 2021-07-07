use bevy::{prelude::*, tasks::ComputeTaskPool, utils::Instant};
use bevy_ecs_tilemap::prelude::*;

use super::{MapData, MapGeneratedEvent, TileType, TILE_BATCH_SIZE, Z_LEVELS};

// TODO
// * add darker shade to hidden tiles
// * use this to figure out which corner piece to use
//      let neighbors = map_query.get_tile_neighbors(*pos, 0u16, tile_parent.layer_id);
// * merge MapRendererData and MapGeneratorData??

pub fn update_layer_visibility(
    mut tile_query: Query<(&mut Tile, &TileParent)>,
    mut chunk_query: Query<&mut Chunk>,
    mut map_data: ResMut<MapData>,
    pool: Res<ComputeTaskPool>,
) {
    if !map_data.is_changed() {
        return;
    }
    info!("updating layer visibility...");
    let start = Instant::now();

    fn visibility_needs_update(z_level: u16, map_data: &ResMut<MapData>) -> Option<bool> {
        let is_layer_visible = map_data.visible_layers[z_level as usize];
        if is_layer_visible && z_level > map_data.current_z_level {
            Some(false)
        } else if !is_layer_visible && z_level <= map_data.current_z_level {
            Some(true)
        } else {
            None
        }
    }

    tile_query.par_for_each_mut(&pool, TILE_BATCH_SIZE, |(mut tile, tile_parent)| {
        if let Some(visible) = visibility_needs_update(tile_parent.layer_id, &map_data) {
            tile.visible = visible;
        }
    });

    for mut chunk in chunk_query.iter_mut() {
        if visibility_needs_update(chunk.settings.layer_id, &map_data).is_some() {
            chunk.needs_remesh = true;
        }
    }

    for z_level in 0..Z_LEVELS {
        if let Some(visible) = visibility_needs_update(z_level, &map_data) {
            map_data.visible_layers[z_level as usize] = visible;
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
    map_generator_data: Res<MapData>,
    mut events: EventReader<MapGeneratedEvent>,
) {
    if events.iter().count() == 0 {
        return;
    }
    info!("setting map textures...");
    let start = Instant::now();

    tile_query.par_for_each_mut(&pool, TILE_BATCH_SIZE, |(mut tile, tile_parent, pos)| {
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
