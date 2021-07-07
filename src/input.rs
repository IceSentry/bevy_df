use crate::camera::CameraData;
use bevy::{input::mouse::MouseWheel, prelude::*};

const CAMERA_SPEED: f32 = 500.;

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
    if keyboard_input.pressed(KeyCode::LShift) {
        camera.movement_strength = CAMERA_SPEED * 2.0;
    } else {
        camera.movement_strength = CAMERA_SPEED;
    }
}

pub fn mouse_wheel(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera: ResMut<CameraData>,
    mut map_data: ResMut<crate::map::MapData>,
) {
    for event in mouse_wheel_events.iter() {
        if keyboard_input.pressed(KeyCode::LControl) {
            let new_z_level = (map_data.current_z_level as i32 - event.y as i32).clamp(0, 20);
            map_data.current_z_level = new_z_level as u16;
        } else {
            camera.scale -= event.y;
            if camera.scale < 1.0 {
                camera.scale = 1.0;
            }
        }
    }
}
