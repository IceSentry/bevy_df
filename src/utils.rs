use bevy::{math::Vec2, prelude::*, window::Window};
use std::ops::Sub;

#[allow(unused)]
pub fn lerp<T: num::Float + Sub>(a: T, b: T, v: T) -> T {
    (T::one() - v) * a + b * v
}

#[allow(unused)]
pub fn inverse_lerp<T: num::Float + Sub>(a: T, b: T, v: T) -> T {
    (v - a) / (b - a)
}

/// Transforms a point in world coordinates to an isometric projection
/// WARN only works with a single layer, doesn't take into accound any z-levels
pub fn world_to_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x / tile_width) + (-pos.y / tile_height);
    let y = (-pos.y / tile_height) - (pos.x / tile_width);
    Vec2::new(x.floor(), y.floor())
}

/// Transforms a point in isometric projection to world coordinates
pub fn iso_to_world(pos: &Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    Vec2::new(x, -y)
}

pub fn cursor_to_world(window: &Window, camera_transform: &Transform, scale: f32) -> Option<Vec4> {
    if let Some(cursor_position) = window.cursor_position() {
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let cursor_position = cursor_position - window_size / 2.0;

        let pos_world =
            camera_transform.compute_matrix() * cursor_position.extend(0.0).extend(1.0) * scale; // The scale doesn't work properly if you move the camera
        Some(pos_world)
    } else {
        None
    }
}

/// based on this gdc talk <https://www.youtube.com/watch?v=LWFzPP8ZbdU>
#[allow(unused)]
pub fn squirrel_noise(position: i32, seed: u32) -> u32 {
    const BIT_NOISE1: u32 = 0x68E31DA4; // 1101 0001 1100 0110 0011 1011 0100 1000
    const BIT_NOISE2: u32 = 0xB5297A4D; // 1011 0101 0010 1001 0111 1010 0100 1101
    const BIT_NOISE3: u32 = 0x1B56C4E9; // 1101 1010 1011 0110 0010 0111 0100 1000

    let mut bits = position as u32;
    bits = bits.wrapping_mul(BIT_NOISE1);
    bits = bits.wrapping_add(seed);
    bits ^= bits >> 8;
    bits = bits.wrapping_add(BIT_NOISE2);
    bits ^= bits << 8;
    bits = bits.wrapping_mul(BIT_NOISE3);
    bits ^= bits >> 8;
    bits
}
