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
