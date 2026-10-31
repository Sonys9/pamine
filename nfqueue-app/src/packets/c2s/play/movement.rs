//! Player and vehicle movement.

use pcore::{Decode, decoder::VarInt};
use pcore_gen::Encode;

#[derive(Decode, Encode, Debug, Clone)]
pub struct ConfirmTeleportation {
    pub teleport_id: VarInt,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetPlayerPositionAndRotation {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetPlayerOnGround {
    pub on_ground: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct MoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct PaddleBoat {
    pub left_paddle_turning: bool,
    pub right_paddle_turning: bool,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct PlayerCommand {
    pub entity_id: VarInt,
    pub action_id: VarInt,
    pub jump_boost: VarInt,
}
