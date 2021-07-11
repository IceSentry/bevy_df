use crate::{
    camera::{MainCamera, SCALE},
    map::{
        CurrentZLevel, MapData, Tile, TileType, TilesToUpdate, HEIGHT, TILE_HEIGHT, TILE_WIDTH,
        WIDTH,
    },
    utils::{cursor_to_world, iso_to_world, world_to_iso},
};
use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec4Swizzles,
    prelude::*,
};

// TODO
// * maybe create a list of selected tiles in one system and update the map in another system
// * instead of just moving the selector around, maybe spawn it when clicked
//   and remove any other existing selector not part of the current selection
// * support multi selection

// WARN there's something broken when trying to select tiles at the edge

pub struct SelectorPlugin;

impl Plugin for SelectorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(selector_setup.system())
            .add_system(selector.system());
    }
}

struct Selector;

fn selector(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    map_data: Res<MapData>,
    current_z_level: Res<CurrentZLevel>,
    mut tiles: ResMut<TilesToUpdate>,
    mut queries: QuerySet<(
        Query<&Transform, With<MainCamera>>,
        Query<&mut Transform, With<Selector>>,
    )>,
) {
    for event in mouse_button_input_events.iter() {
        if let ElementState::Pressed = event.state {
            let window = windows.get_primary().expect("primary window not found");
            let camera_transform = queries.q0().single().expect("main camera not found");
            let cursor_position = cursor_to_world(window, camera_transform, SCALE)
                .expect("cursor_position not found");
            let selected_pos = world_to_iso(
                cursor_position.xy(),
                TILE_WIDTH as f32,
                TILE_HEIGHT as f32 / 2.0,
            );

            for mut selector in queries.q1_mut().iter_mut() {
                let pos = iso_to_world(&selected_pos, TILE_WIDTH as f32, TILE_HEIGHT as f32 / 2.0);
                selector.translation = pos.extend(current_z_level.0 as f32);

                let tile_pos = find_highest_tile(selected_pos, &map_data, current_z_level.0);
                if tile_pos.x <= WIDTH as u32 || tile_pos.y <= HEIGHT as u32 {
                    // TODO check if there's a tile above to make sure we aren't clicking through a tile
                    tiles.0.push((
                        tile_pos,
                        Tile {
                            value: TileType::Air,
                            visible: true,
                        },
                    ));
                }
            }
        }
    }
}

fn find_highest_tile(pos: Vec2, map_data: &MapData, current_z_level: u16) -> UVec3 {
    let mut last_checked_position = pos.as_u32().extend(0);
    let mut out = last_checked_position;
    for _z in 0..current_z_level {
        if let Some(tile) = map_data.get_tile(last_checked_position) {
            if !matches!(tile.value, TileType::Air) {
                out = last_checked_position;
            }
            last_checked_position += UVec3::ONE;
        } else {
            break;
        }
    }
    out
}

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
