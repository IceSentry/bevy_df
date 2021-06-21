use bevy::{prelude::*, render::camera::Camera};

pub struct CameraData {
    pub scale: f32,
    pub direction: Vec3,
    pub movement_strength: f32,
}

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update.system()).insert_resource(CameraData {
            scale: 3.0,
            direction: Vec3::ZERO,
            movement_strength: 500.,
        });
    }
}

pub fn update(
    camera: Res<CameraData>,
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        transform.scale = Vec3::splat(camera.scale);
        if transform.scale.x < 1.0 {
            transform.scale = Vec3::ONE;
        }
        transform.translation += time.delta_seconds() * camera.direction * camera.movement_strength;
    }
}
