use bevy::{prelude::*, render::camera::Camera};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
pub struct CameraData {
    #[inspectable(min = 1.0, max = 10.0)]
    scale: f32,
}

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(InspectorPlugin::<CameraData>::new())
            .add_system(movement.system())
            .add_system(update.system())
            .insert_resource(CameraData { scale: 3.0 });
    }
}

// A simple camera system for moving and zooming the camera.
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let scale = transform.scale.x;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            let scale = scale + 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if keyboard_input.pressed(KeyCode::X) {
            let scale = scale - 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if transform.scale.x < 1.0 {
            transform.scale = Vec3::splat(1.0)
        }

        transform.translation += time.delta_seconds() * direction * 500.;
    }
}

pub fn update(camera: ResMut<CameraData>, mut query: Query<&mut Transform, With<Camera>>) {
    for mut transform in query.iter_mut() {
        transform.scale = Vec3::splat(camera.scale);
        if transform.scale.x < 1.0 {
            transform.scale = Vec3::splat(1.0)
        }
    }
}
