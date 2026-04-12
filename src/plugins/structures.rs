// Void Architect — plugins/structures.rs
// Wall, Turret, Farm placement, durability, turret AI.
// STUB — akan diimplementasi di Sprint 1 (S1-05 hingga S1-08)

use bevy::prelude::*;
use crate::GameState;

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, _placeholder.run_if(in_state(GameState::InRun)));
    }
}

fn _placeholder() {}
