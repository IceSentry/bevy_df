use bevy::{
    diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};

pub(crate) fn performance_ui(egui_context: ResMut<EguiContext>, diagnostics: ResMut<Diagnostics>) {
    let diagnostics = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average)
        .zip(
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(Diagnostic::average),
        );
    let (fps, frame_time) = match diagnostics {
        Some(value) => value,
        None => {
            warn!("Add the `FrameTimeDiagnosticsPlugin` to see the performance editor panel.");
            return;
        }
    };

    egui::Window::new("Performance").show(egui_context.ctx(), |ui| {
        egui::Grid::new("frame time diagnostics").show(ui, |ui| {
            ui.label("FPS");
            ui.label(format!("{:.2}", fps));
            ui.end_row();
            ui.label("Frame Time");
            ui.label(format!("{:.4}", frame_time));
            ui.end_row();
        });
    });
}
