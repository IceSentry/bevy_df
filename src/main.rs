#![allow(clippy::type_complexity)]

use std::collections::VecDeque;

use bevy::{
    diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::texture::FilterMode,
};
use bevy_egui::EguiPlugin;
use bevy_egui::{egui, EguiContext};

mod camera;
mod input;
pub mod map;
mod selector;
mod utils;

const FRAME_TIME_HISTORY_LEN: usize = 100;

struct FrameTimeHistory(VecDeque<f64>);

fn performance_display(
    egui_context: ResMut<EguiContext>,
    diagnostics: ResMut<Diagnostics>,
    mut values: ResMut<FrameTimeHistory>,
) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average);
    let frame_time = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(Diagnostic::value)
        .map(|x| x * 1000.);

    let (fps, frame_time) = match (fps, frame_time) {
        (Some(fps), Some(frame_time)) => (fps, frame_time),
        _ => return,
    };

    values.0.push_back(frame_time);
    if values.0.len() > FRAME_TIME_HISTORY_LEN {
        values.0.pop_front();
    }

    let line = egui::plot::Line::new(egui::plot::Values::from_values_iter(
        values
            .0
            .iter()
            .enumerate()
            .map(|(i, value)| egui::plot::Value::new(i as f64, *value)),
    ));

    egui::Area::new("Performance area")
        .anchor(egui::Align2::LEFT_TOP, [0., 0.])
        .show(egui_context.ctx(), |ui| {
            ui.label(format!("FPS {:.2}", fps));
            ui.end_row();
            ui.label(format!("Frame Time {:.2}ms", frame_time));
            ui.end_row();
            ui.add(
                egui::plot::Plot::new("frame_time_plot")
                    .line(line)
                    .center_x_axis(false)
                    .center_y_axis(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .view_aspect(3.0),
            );
        });
}

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
        .add_plugin(selector::SelectorPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(performance_display.system())
        .insert_resource(FrameTimeHistory(VecDeque::with_capacity(
            FRAME_TIME_HISTORY_LEN,
        )))
        .run();
}
