// Void Architect — plugins/combat.rs
// Hit detection, damage resolution, death events.
// STUB — akan diimplementasi di Sprint 1 (S1-01 hingga S1-13)

use bevy::prelude::*;
use crate::GameState;
use crate::components::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DamageEvent>()
            .add_event::<EnemyDied>()
            .add_event::<VoidCoreDamaged>()
            .add_systems(Update, process_damage_events
                .run_if(in_state(GameState::InRun)));
    }
}

/// Stub: proses DamageEvent → kurangi Health target.
/// Sprint 1 akan tambah: flash effect, death, loot drop, EXP.
fn process_damage_events(
    mut damage_events: EventReader<DamageEvent>,
    mut health_q: Query<&mut Health>,
) {
    for ev in damage_events.read() {
        if let Ok(mut health) = health_q.get_mut(ev.target) {
            health.damage(ev.amount);
        }
    }
}
