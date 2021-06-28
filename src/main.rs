#![allow(clippy::type_complexity)]

use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec4Swizzles,
    prelude::*,
    render::texture::FilterMode,
};
use camera::MainCamera;
use map::map_renderer::MapRendererData;

use crate::{
    camera::SCALE,
    map::{TILE_HEIGHT, TILE_WIDTH},
};

mod camera;
mod input;
pub mod map;
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

fn mouse(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    mut queries: QuerySet<(
        Query<&Transform, With<MainCamera>>,
        Query<&mut Transform, With<Selector>>,
    )>,
    map_data: Res<MapRendererData>,
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
                    selector.translation.z = map_data.current_z_level as f32 + 1.;
                }
            }
        }
    }
}

/// Transforms a point in world coordinates to an isometric projection
/// WARN only works with a single layer, doesn't take into accound any z-levels
fn world_to_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x / tile_width) + (-pos.y / tile_height);
    let y = (-pos.y / tile_height) - (pos.x / tile_width);
    Vec2::new(x.floor(), y.floor())
}

/// Transform a point in isometric projection to world coordinates
fn iso_to_world(pos: &Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    Vec2::new(x, -y)
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
        // .add_plugin(LogDiagnosticsPlugin::filtered(vec![
        //     FrameTimeDiagnosticsPlugin::FPS,
        //     FrameTimeDiagnosticsPlugin::FRAME_TIME,
        // ]))
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(camera::CameraControlPlugin)
        .add_plugin(map::MapPlugin)
        .add_plugin(input::InputPlugin)
        .add_startup_system(selector_setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(mouse.system())
        .run();
}
