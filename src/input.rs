use crate::camera::CameraData;
use bevy::{input::mouse::MouseWheel, prelude::*};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(movement.system())
            .add_system(mouse_wheel.system());
    }
}

pub fn movement(keyboard_input: Res<Input<KeyCode>>, mut camera: ResMut<CameraData>) {
    camera.direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::A) {
        camera.direction -= Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::D) {
        camera.direction += Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::W) {
        camera.direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::S) {
        camera.direction -= Vec3::Y;
    }
}

pub fn mouse_wheel(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera: ResMut<CameraData>,
    mut map_state: ResMut<crate::map::map_renderer::MapRendererData>,
) {
    for event in mouse_wheel_events.iter() {
        if keyboard_input.pressed(KeyCode::LControl) {
            let new_z_level = (map_state.current_z_level as i32 + -event.y as i32).clamp(0, 20);
            map_state.current_z_level = new_z_level as u16;
        } else {
            camera.scale += -event.y as i32;
        }
    }
}
