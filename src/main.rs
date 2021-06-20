use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::texture::FilterMode,
};

use simplelog::{self, ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

mod camera;
mod input;
pub mod map;

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.sampler.min_filter = FilterMode::Nearest;
            }
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
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
            FrameTimeDiagnosticsPlugin::FRAME_TIME,
        ]))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(camera::CameraControlPlugin)
        .add_plugin(map::MapPlugin)
        .add_plugin(input::InputPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(set_texture_filters_to_nearest.system())
        .run();
}
