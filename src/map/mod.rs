use anyhow::{bail, Result};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use noise::{Seedable, SuperSimplex};

use self::{
    generator::{generate_map, NoiseSettings},
    renderer::{set_map_textures, update_layer_visibility, update_tiles},
};

pub mod generator;
pub mod renderer;

// MAP & CHUNKS
pub const CHUNK_WIDTH: u32 = 64;
pub const CHUNK_HEIGHT: u32 = 64;
pub const MAP_WIDTH: u32 = 5;
pub const MAP_HEIGHT: u32 = 5;
pub const WIDTH: usize = MAP_WIDTH as usize * CHUNK_WIDTH as usize;
pub const HEIGHT: usize = MAP_HEIGHT as usize * CHUNK_HEIGHT as usize;
pub const Z_LEVELS: u16 = 20;
pub const ELEVATION_MULTIPLIER: f32 = 1.0 / Z_LEVELS as f32;
pub const TOTAL_TILE_COUNT: usize = WIDTH * HEIGHT * Z_LEVELS as usize;
pub const TILE_BATCH_SIZE: usize = TOTAL_TILE_COUNT / 6;

// TILE
pub const TILE_WIDTH: usize = 32;
pub const TILE_HEIGHT: usize = 32;

pub const TEXTURE_WIDTH: usize = 32 * 6;
pub const TEXTURE_HEIGHT: usize = 32;

pub struct MapGeneratedEvent;

pub struct CurrentZLevel(pub u16);

// TODO consider using a queue
// Maybe tag existing tiles instead and query tiles with the tag
pub struct TilesToUpdate(pub Vec<(UVec3, Tile)>);

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
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.data.get(y * WIDTH + x)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, new_tile: Tile) -> Result<()> {
        if let Some(elem) = self.data.get_mut(y * WIDTH + x) {
            *elem = new_tile;
            Ok(())
        } else {
            bail!("tile out of bounds")
        }
    }
}

pub struct MapData {
    pub layers: Vec<Layer>,
}

pub struct VisibleLayers(Vec<bool>);

impl VisibleLayers {
    pub fn new(z_levels: usize) -> Self {
        Self(vec![true; z_levels])
    }
}

impl MapData {
    pub fn new(width: usize, height: usize, z_levels: usize) -> Self {
        Self {
            layers: vec![Layer::new(width, height); z_levels],
        }
    }

    pub fn get_tile(&self, pos: UVec3) -> Option<&Tile> {
        self.layers
            .get(pos.z as usize)
            .and_then(|l| l.get_tile(pos.x as usize, pos.y as usize))
    }

    pub fn set_tile(&mut self, pos: UVec3, new_tile: Tile) -> Result<()> {
        if let Some(l) = self.layers.get_mut(pos.z as usize) {
            l.set_tile(pos.x as usize, pos.y as usize, new_tile)?;
            Ok(())
        } else {
            bail!("tile out of bounds")
        }
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TilemapPlugin)
            .add_plugin(InspectorPlugin::<NoiseSettings>::new())
            .add_event::<MapGeneratedEvent>()
            .add_startup_system(startup.system())
            .add_system(generate_map.system())
            .add_system(set_map_textures.system())
            .add_system(update_layer_visibility.system())
            .add_system(update_tiles.system());
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let texture_handle = asset_server.load("iso_tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut layer_settings = LayerSettings::new(
        UVec2::new(MAP_WIDTH, MAP_HEIGHT),
        UVec2::new(CHUNK_WIDTH, CHUNK_HEIGHT),
        Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32),
        Vec2::new(TEXTURE_WIDTH as f32, TEXTURE_HEIGHT as f32),
    );
    layer_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    // This is needed because the tiles have a depth that isn't aligned to the grid
    layer_settings.grid_size = Vec2::new(TILE_HEIGHT as f32, TILE_HEIGHT as f32 / 2.0);

    for layer_id in 0..Z_LEVELS {
        let layer_entity = LayerBuilder::<TileBundle>::new_batch(
            &mut commands,
            layer_settings,
            &mut meshes,
            material_handle.clone(),
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
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());

    let noise_fn = SuperSimplex::new().set_seed(42);
    commands.insert_resource(noise_fn);

    commands.insert_resource(MapData {
        layers: vec![Layer::new(WIDTH, HEIGHT); Z_LEVELS as usize],
    });

    commands.insert_resource(VisibleLayers::new(Z_LEVELS as usize));

    commands.insert_resource(CurrentZLevel(Z_LEVELS));

    commands.insert_resource(TilesToUpdate(vec![]));
}
