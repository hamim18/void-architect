// Void Architect — plugins/enemies.rs
// Enemy spawning, AI behavior, adaptation tracking.
// STUB — akan diimplementasi di Sprint 1 (S1-09 hingga S1-12) dan Sprint 3 (S3-04)

use bevy::prelude::*;
use crate::GameState;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, _placeholder.run_if(in_state(GameState::InRun)));
    }
}

fn _placeholder() {}
