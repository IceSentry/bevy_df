use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use noise::{Seedable, SuperSimplex};

use crate::map::map_generator::MapGeneratorData;

use self::{
    map_generator::{generate_map, NoiseSettings},
    map_renderer::{set_map_textures, update_map_state, MapRendererData},
};

mod map_generator;
pub mod map_renderer;

// MAP & CHUNKS
pub const CHUNK_WIDTH: u32 = 64;
pub const CHUNK_HEIGHT: u32 = 64;
pub const MAP_WIDTH: u32 = 3;
pub const MAP_HEIGHT: u32 = 3;
pub const WIDTH: usize = MAP_WIDTH as usize * CHUNK_WIDTH as usize;
pub const HEIGHT: usize = MAP_HEIGHT as usize * CHUNK_HEIGHT as usize;
pub const Z_LEVELS: u16 = 20;
pub const ELEVATION_MULTIPLIER: f32 = 1.0 / Z_LEVELS as f32;

// TILE
pub const TILE_WIDTH: usize = 32;
pub const TILE_HEIGHT: usize = 32;

pub const TEXTURE_WIDTH: usize = 32 * 6;
pub const TEXTURE_HEIGHT: usize = 32;

pub struct MapGeneratedEvent;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TilemapPlugin)
            .add_plugin(InspectorPlugin::<NoiseSettings>::new())
            .add_event::<MapGeneratedEvent>()
            .add_startup_system(startup.system())
            .add_system(generate_map.system())
            .add_system(set_map_textures.system())
            .add_system(update_map_state.system());
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Load texture
    let texture_handle = asset_server.load("iso_tiles.png");
    let tile_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut layer_settings = LayerSettings::new(
        UVec2::new(MAP_WIDTH, MAP_HEIGHT),
        UVec2::new(CHUNK_WIDTH, CHUNK_HEIGHT),
        Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32),
        Vec2::new(TEXTURE_WIDTH as f32, TEXTURE_HEIGHT as f32),
    );
    // This is needed because the tiles have a depth that isn't aligned to the grid
    layer_settings.grid_size = Vec2::new(TILE_HEIGHT as f32, TILE_HEIGHT as f32 / 2.0);
    layer_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    for layer_id in 0..Z_LEVELS {
        let layer_entity = LayerBuilder::<TileBundle>::new_batch(
            &mut commands,
            layer_settings,
            &mut meshes,
            tile_handle.clone(),
            0u16,
            layer_id,
            None,
            move |_| Some(TileBundle::default()),
        );
        commands.entity(layer_entity).insert(Transform::from_xyz(
            0.0,
            (layer_id as f32) * (TILE_HEIGHT as f32 / 2.0), // offset each z_level to stack them
            layer_id as f32,
        ));
        map.add_layer(&mut commands, layer_id, layer_entity);
    }

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(
            0.0,
            (TILE_HEIGHT as f32 / 2.0) * (HEIGHT as f32 / 2.0),
            0.0,
        ))
        .insert(GlobalTransform::default());

    let noise_fn = SuperSimplex::new().set_seed(42);
    commands.insert_resource(noise_fn);

    commands.insert_resource(MapRendererData {
        visible_layers: vec![true; Z_LEVELS as usize],
        current_z_level: Z_LEVELS,
    });

    commands.insert_resource(MapGeneratorData::new(WIDTH, HEIGHT, Z_LEVELS as usize));
}
