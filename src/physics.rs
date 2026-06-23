use bevy::prelude::Vec2;

pub const G: f32 = 6_000.0;

pub fn gravitational_acceleration(pos: Vec2, attractor: Vec2, mass: f32) -> Vec2 {
    let delta = attractor - pos;
    let dist_sq = delta.length_squared().max(400.0); // clamp minimum distance
    let dist = dist_sq.sqrt();
    delta / dist * (G * mass / dist_sq)
}
