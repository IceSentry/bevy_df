use std::ops::Sub;

use bevy::math::Vec2;

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

/// Transform a point in isometric projection to world coordinates
pub fn iso_to_world(pos: &Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    Vec2::new(x, -y)
}
