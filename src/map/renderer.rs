use bevy::{math::Vec3Swizzles, prelude::*, tasks::ComputeTaskPool, utils::Instant};
use bevy_ecs_tilemap::prelude::*;

use crate::map::CurrentZLevel;

use super::{
    MapData, MapGeneratedEvent, TileType, TilesToUpdate, VisibleLayers, TILE_BATCH_SIZE, Z_LEVELS,
};

// TODO
// * add darker shade to hidden tiles
// * use this to figure out which corner piece to use
//      let neighbors = map_query.get_tile_neighbors(*pos, 0u16, tile_parent.layer_id);
// * merge MapRendererData and MapGeneratorData??

pub fn update_layer_visibility(
    mut tile_query: Query<(&mut Tile, &TileParent)>,
    mut chunk_query: Query<&mut Chunk>,
    mut visible_layers: ResMut<VisibleLayers>,
    current_z_level: Res<CurrentZLevel>,
    pool: Res<ComputeTaskPool>,
) {
    if !current_z_level.is_changed() {
        return;
    }

    info!("updating layer visibility...");
    let start = Instant::now();

    fn visibility_needs_update(
        z_level: u16,
        visible_layers: &VisibleLayers,
        current_z_level: &CurrentZLevel,
    ) -> Option<bool> {
        let is_layer_visible = visible_layers.0[z_level as usize];
        if is_layer_visible && z_level > current_z_level.0 {
            Some(false)
        } else if !is_layer_visible && z_level <= current_z_level.0 {
            Some(true)
        } else {
            None
        }
    }

    tile_query.par_for_each_mut(&pool, TILE_BATCH_SIZE, |(mut tile, tile_parent)| {
        if let Some(visible) =
            visibility_needs_update(tile_parent.layer_id, &visible_layers, &current_z_level)
        {
            tile.visible = visible;
        }
    });

    for mut chunk in chunk_query.iter_mut() {
        if visibility_needs_update(chunk.settings.layer_id, &visible_layers, &current_z_level)
            .is_some()
        {
            chunk.needs_remesh = true;
        }
    }

    for z_level in 0..Z_LEVELS {
        if let Some(visible) = visibility_needs_update(z_level, &visible_layers, &current_z_level) {
            visible_layers.0[z_level as usize] = visible;
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
    map_data: Res<MapData>,
    mut events: EventReader<MapGeneratedEvent>,
) {
    if events.iter().count() == 0 {
        return;
    }
    info!("setting map textures...");
    let start = Instant::now();

    tile_query.par_for_each_mut(&pool, TILE_BATCH_SIZE, |(mut tile, tile_parent, pos)| {
        let tile_data = &map_data
            .get_tile(pos.extend(tile_parent.layer_id as u32))
            .expect("Tile is out of bounds");

        tile.texture_index = match tile_data.value {
            TileType::Air => 1,
            TileType::Water => 2,
            TileType::Grass => 3,
            TileType::Dirt => 4,
            TileType::Rock => 5,
        };
    });

    for mut chunk in chunk_query.iter_mut() {
        chunk.needs_remesh = true;
    }
    info!("setting map textures...done elapsed: {:?}", start.elapsed());
}

pub fn update_tiles(
    mut map_data: ResMut<MapData>,
    mut map_query: MapQuery,
    mut tile_query: Query<&mut bevy_ecs_tilemap::Tile>,
    mut tiles: ResMut<TilesToUpdate>,
) {
    for (tile_pos, tile_data) in &tiles.0 {
        map_data
            .set_tile(*tile_pos, *tile_data)
            .expect("tile out of bounds");
        let tile_entity = map_query
            .get_tile_entity(tile_pos.xy(), 0u16, tile_pos.z as u16)
            .expect("no tile entity found");
        if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
            tile.texture_index = match tile_data.value {
                TileType::Air => 1,
                TileType::Water => 2,
                TileType::Grass => 3,
                TileType::Dirt => 4,
                TileType::Rock => 5,
            };
        }
        // TODO cache chunks that needs updating
        map_query.notify_chunk_for_tile(tile_pos.xy(), 0u16, tile_pos.z as u16);
    }
    tiles.0.clear();
}
