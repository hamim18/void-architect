// Void Architect — plugins/colony.rs
// NPC roles, hunger system, morale, population events.
// STUB — akan diimplementasi di Sprint 2 (S2-01 hingga S2-09)

use bevy::prelude::*;
use crate::GameState;
use crate::components::NpcLost;

pub struct ColonyPlugin;

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<NpcLost>()
            .add_systems(Update, _placeholder.run_if(in_state(GameState::InRun)));
    }
}

fn _placeholder() {}
