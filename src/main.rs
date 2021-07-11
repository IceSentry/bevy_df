#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec4Swizzles,
    prelude::*,
    render::texture::FilterMode,
};
use bevy_ecs_tilemap::MapQuery;
use bevy_egui::EguiPlugin;
use camera::MainCamera;
use map::{MapData, Tile, TileType, Z_LEVELS};
use performance_ui::performance_ui;
use utils::{iso_to_world, world_to_iso};

use crate::{
    camera::SCALE,
    map::{MapGeneratedEvent, HEIGHT, TILE_HEIGHT, TILE_WIDTH, WIDTH},
};

mod camera;
mod input;
pub mod map;
mod performance_ui;
mod utils;

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    // This helps remove lines that appears when camera is far away
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.sampler.min_filter = FilterMode::Nearest;
            }
        }
    }
}

fn selector(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    mut map_data: ResMut<MapData>,
    mut map_query: MapQuery,
    mut tile_query: Query<&mut bevy_ecs_tilemap::Tile>,
    mut queries: QuerySet<(
        Query<&Transform, With<MainCamera>>,
        Query<&mut Transform, With<Selector>>,
    )>,
) {
    for event in mouse_button_input_events.iter() {
        if let ElementState::Pressed = event.state {
            let window = windows.get_primary().expect("No window found :(");
            if let Some(cursor_position) = window.cursor_position() {
                let window_size = Vec2::new(window.width() as f32, window.height() as f32);

                // the default orthographic projection is in pixels from the center;
                // just undo the translation
                let cursor_position = cursor_position - window_size / 2.0;

                let camera_transform = queries.q0().single().expect("No camera found :(");
                let pos_world = camera_transform.compute_matrix()
                    * cursor_position.extend(0.0).extend(1.0)
                    * SCALE; // The scale doesn't work properly if you move the camera

                let selected =
                    world_to_iso(pos_world.xy(), TILE_WIDTH as f32, TILE_HEIGHT as f32 / 2.0);

                for mut selector in queries.q1_mut().iter_mut() {
                    let pos = iso_to_world(&selected, TILE_WIDTH as f32, TILE_HEIGHT as f32 / 2.0);
                    selector.translation.x = pos.x;
                    selector.translation.y = pos.y - ((TILE_HEIGHT as f32 / 2.0) / 2.0);
                    selector.translation.z = map_data.current_z_level as f32 + TILE_HEIGHT as f32;

                    // find highest tile
                    let (new_pos, z) = calculate_z(selected, &map_data);
                    if new_pos.x <= WIDTH as f32 || new_pos.y <= HEIGHT as f32 {
                        // TODO check if there's a tile above to make sure we aren't clicking through a tile
                        map_data.layers[z].set_tile(
                            new_pos.x as usize,
                            new_pos.y as usize,
                            Tile {
                                value: TileType::Air,
                                visible: true,
                            },
                        );
                        let tile_entity = map_query
                            .get_tile_entity(new_pos.as_u32(), 0u16, z as u16)
                            .expect("no tile found at location");
                        if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                            // TODO use from or something to convert TileType
                            tile.texture_index = 1; // Air
                        }
                        map_query.notify_chunk_for_tile(new_pos.as_u32(), 0u16, z as u16);
                    }
                }
            }
        }
    }
}

fn calculate_z(pos: Vec2, map_data: &ResMut<MapData>) -> (Vec2, usize) {
    let mut output = 0;
    let mut check_pos = pos;
    let mut output_pos = check_pos;
    for z in 0..=map_data.current_z_level {
        if z == Z_LEVELS {
            break;
        }
        if check_pos.x >= WIDTH as f32 || check_pos.y >= HEIGHT as f32 {
            break;
        }
        let tile_type = map_data.layers[z as usize]
            .get_tile(check_pos.x.floor() as usize, check_pos.y.floor() as usize)
            .value;
        match tile_type {
            TileType::Air => (),
            _ => {
                output = z as usize;
                output_pos = Vec2::new(check_pos.x.floor(), check_pos.y.floor())
            }
        }
        check_pos += Vec2::new(1., 1.);
    }

    (output_pos, output)
}

struct Selector;

fn selector_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("iso_select.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .insert(Selector);
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: String::from("bevy_df"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(camera::CameraControlPlugin)
        .add_plugin(map::MapPlugin)
        .add_plugin(input::InputPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(set_texture_filters_to_nearest.system())
        .add_startup_system(selector_setup.system())
        .add_system(selector.system())
        .add_system(performance_ui.system())
        .run();
}
