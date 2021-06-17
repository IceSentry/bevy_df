use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use noise::{NoiseFn, Seedable, SuperSimplex};
use simplelog::{self, ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

mod camera;

struct NeedUpdate(bool);

const CHUNK_WIDTH: u32 = 64;
const CHUNK_HEIGHT: u32 = 64;
const MAP_WIDTH: u32 = 4;
const MAP_HEIGHT: u32 = 4;
const WIDTH: usize = MAP_WIDTH as usize * CHUNK_WIDTH as usize;
const HEIGHT: usize = MAP_HEIGHT as usize * CHUNK_HEIGHT as usize;

struct MapData {
    data: Vec<f64>,
}

struct MapGeneratedEvent;

#[derive(Inspectable, Default)]
struct NoiseSettings {
    #[inspectable(min = 1.0, max = 10.0, speed = 0.1)]
    frequency: f64,
    #[inspectable(visual, min = Vec2::splat(-2.0), max = Vec2::splat(2.0))]
    offset: Vec2,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let layer_settings = LayerSettings::new(
        UVec2::new(MAP_WIDTH, MAP_HEIGHT),
        UVec2::new(CHUNK_WIDTH, CHUNK_HEIGHT),
        Vec2::new(16.0, 16.0),
        Vec2::new(96.0, 256.0),
    );

    let center = layer_settings.get_pixel_center();

    // Chunk sizes of 64x64 seem optimal for meshing updates.
    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16, None);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.for_each_tiles_mut(|tile_entity, tile_data| {
        *tile_data = Some(TileBundle::default());
        // Tile entity might not exist at this point so you'll need to create it.
        if tile_entity.is_none() {
            *tile_entity = Some(commands.spawn().id());
        }
        commands.entity(tile_entity.unwrap());
    });

    map_query.build_layer(&mut commands, layer_builder, material_handle);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());

    commands.insert_resource(NeedUpdate(true));

    let noise_fn = SuperSimplex::new().set_seed(42);
    commands.insert_resource(noise_fn);

    commands.insert_resource(MapData {
        data: vec![0.0; WIDTH * HEIGHT],
    });
}

fn generate_map(
    noise: Res<SuperSimplex>,
    noise_settings: Res<NoiseSettings>,
    mut map: ResMut<MapData>,
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
            let point = Vec2::new(current_x as f32, current_y as f32)
                * noise_settings.frequency as f32
                + noise_settings.offset;
            let elevation = noise.get([point.x as f64, point.y as f64, 0.0]);
            let elevation = (elevation + 1.0) * 0.5;
            let map_idx = y * WIDTH + x;
            map.data[map_idx] = elevation;
        }
    }
    event.send(MapGeneratedEvent);
}

fn draw_map(
    mut tile_query: Query<(&mut Tile, &UVec2)>,
    mut map_query: MapQuery,
    map: Res<MapData>,
    mut events: EventReader<MapGeneratedEvent>,
) {
    for _ in events.iter() {
        for (mut tile, pos) in tile_query.iter_mut() {
            let i = pos.y as usize * WIDTH + pos.x as usize;
            let elevation = map.data[i];
            tile.texture_index = if elevation < 0.4 {
                1 // water
            } else if elevation < 0.6 {
                0 // grass
            } else if elevation < 0.8 {
                3 // rock
            } else {
                5 // snow
            };
            map_query.notify_chunk_for_tile(*pos, 0u16, 0u16);
        }
    }
}

fn main() {
    TermLogger::new(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    );

    App::build()
        .insert_resource(WindowDescriptor {
            title: String::from("bevy_df"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(InspectorPlugin::<NoiseSettings>::new())
        .add_plugin(camera::CameraControlPlugin)
        .add_event::<MapGeneratedEvent>()
        .add_startup_system(startup.system())
        .add_system(camera::movement.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(generate_map.system())
        .add_system(draw_map.system())
        .run();
}
