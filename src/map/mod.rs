use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use noise::{Seedable, SuperSimplex};

use crate::map::map_generator::MapGeneratorData;

use self::{
    map_generator::{generate_map, NoiseSettings},
    map_renderer::{draw_map, update_map_state, MapRendererData},
};

mod map_generator;
pub mod map_renderer;

pub const CHUNK_WIDTH: u32 = 64;
pub const CHUNK_HEIGHT: u32 = 64;
pub const MAP_WIDTH: u32 = 2;
pub const MAP_HEIGHT: u32 = 2;
pub const WIDTH: usize = MAP_WIDTH as usize * CHUNK_WIDTH as usize;
pub const HEIGHT: usize = MAP_HEIGHT as usize * CHUNK_HEIGHT as usize;
pub const Z_LEVELS: u16 = 20;
pub const ELEVATION_MULTIPLIER: f32 = 1.0 / Z_LEVELS as f32;

pub struct MapGeneratedEvent;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(TilemapPlugin)
            .add_plugin(InspectorPlugin::<NoiseSettings>::new())
            .add_event::<MapGeneratedEvent>()
            .add_startup_system(startup.system())
            .add_system(generate_map.system())
            .add_system(draw_map.system())
            .add_system(update_map_state.system());
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles2.png");
    let tile_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let layer_settings = LayerSettings::new(
        UVec2::new(MAP_WIDTH, MAP_HEIGHT),
        UVec2::new(CHUNK_WIDTH, CHUNK_HEIGHT),
        Vec2::new(16.0, 16.0),
        Vec2::new(128.0, 32.0),
    );
    commands.insert_resource(layer_settings);

    for i in 0..Z_LEVELS {
        let layer_builder = build_layer(&mut commands, layer_settings, &mut map, i);
        map_query.build_layer(&mut commands, layer_builder, tile_handle.clone());
    }

    let center = layer_settings.get_pixel_center();
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());

    let noise_fn = SuperSimplex::new().set_seed(42);
    commands.insert_resource(noise_fn);

    commands.insert_resource(MapRendererData {
        visible_layers: vec![true; Z_LEVELS as usize],
        current_z_level: Z_LEVELS,
    });

    commands.insert_resource(MapGeneratorData::new(WIDTH, HEIGHT, Z_LEVELS as usize));
}

fn build_layer(
    mut commands: &mut Commands,
    layer_settings: LayerSettings,
    map: &mut Map,
    layer_id: u16,
) -> LayerBuilder<TileBundle> {
    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, layer_id, None);
    map.add_layer(&mut commands, layer_id, layer_entity);
    layer_builder.for_each_tiles_mut(|tile_entity, tile_data| {
        *tile_data = Some(TileBundle::default());
        if tile_entity.is_none() {
            *tile_entity = Some(commands.spawn().id());
        }
        commands.entity(tile_entity.unwrap());
    });
    layer_builder
}
