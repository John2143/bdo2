use bevy::prelude::*;

#[derive(Component)]
pub struct PhysicsProperties {
    pub movement_speed_ground: f32,
    pub movement_speed_air: f32,
    pub movement_acceleration: f32,
    pub dash_cooldown: f64,
}

#[derive(Component)]
pub struct Physics {
    pub gravity_func: fn(f32, f32) -> f32,
    pub velocity: Vec3,
    pub walking_velocity: Vec2,
    pub dash_velocity: Vec3,
    pub last_jump: f64,
    pub last_dash: f64,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            gravity_func: |_, _| 9.8,
            velocity: Vec3::ZERO,
            walking_velocity: Vec2::ZERO,
            last_jump: 0.0,
            last_dash: 0.0,
            dash_velocity: Vec3::ZERO,
        }
    }
}

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum NetworkingAction {
    Print(String),
    Location(Quat, Vec3),
    Heartbeat,
}
