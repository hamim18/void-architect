// Void Architect — plugins/phase.rs
// Day/Night phase timer, PhaseChanged event dispatch, wave trigger. [S0-05]

use bevy::prelude::*;

use crate::components::*;
use crate::GameState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const DAY_DURATION: f32 = 180.0;   // 3 menit
pub const NIGHT_DURATION: f32 = 120.0; // 2 menit

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PhasePlugin;

impl Plugin for PhasePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PhaseTimer::default())
            .add_event::<PhaseChanged>()
            .add_systems(OnEnter(GameState::InRun), reset_phase_timer)
            .add_systems(Update, (
                tick_phase_timer,
                // Debug: print phase info ke console (dibuang di release)
                #[cfg(debug_assertions)]
                debug_phase_log,
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Reset timer ke Day fase saat run dimulai.
fn reset_phase_timer(mut timer: ResMut<PhaseTimer>) {
    *timer = PhaseTimer::default();
}

/// Countdown phase timer. Saat habis, transisi ke fase berikutnya.
fn tick_phase_timer(
    mut timer: ResMut<PhaseTimer>,
    mut phase_events: EventWriter<PhaseChanged>,
    time: Res<Time>,
) {
    timer.remaining -= time.delta_seconds();

    if timer.remaining <= 0.0 {
        // Transisi fase
        match timer.phase {
            Phase::Day => {
                // Day → Night
                timer.phase = Phase::Night;
                timer.remaining = NIGHT_DURATION;
                timer.wave_num += 1;

                phase_events.send(PhaseChanged {
                    new_phase: Phase::Night,
                    day: timer.day,
                    wave_num: timer.wave_num,
                });
            }
            Phase::Night => {
                // Night → Day (next day)
                timer.phase = Phase::Day;
                timer.remaining = DAY_DURATION;
                timer.day += 1;

                phase_events.send(PhaseChanged {
                    new_phase: Phase::Day,
                    day: timer.day,
                    wave_num: timer.wave_num,
                });
            }
        }
    }
}

/// Debug log — hanya aktif saat development build.
#[cfg(debug_assertions)]
fn debug_phase_log(
    timer: Res<PhaseTimer>,
    mut last_second: Local<f32>,
) {
    let current_second = timer.remaining.floor();
    if current_second != *last_second {
        *last_second = current_second;
        // Hanya log setiap 10 detik agar tidak spam
        if current_second as u32 % 10 == 0 {
            let phase_name = match timer.phase {
                Phase::Day => "DAY",
                Phase::Night => "NIGHT",
            };
            bevy::log::info!(
                "[Phase] Day {} | {} | {:.0}s remaining | Wave {}",
                timer.day, phase_name, timer.remaining, timer.wave_num
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helper — dipakai plugin lain
// ---------------------------------------------------------------------------

/// Cek apakah saat ini fase siang.
pub fn is_day(timer: &PhaseTimer) -> bool {
    timer.phase == Phase::Day
}

/// Cek apakah saat ini malam dan wave-nya adalah boss wave (setiap 5 malam).
pub fn is_boss_wave(timer: &PhaseTimer) -> bool {
    timer.phase == Phase::Night && timer.wave_num % 5 == 0 && timer.wave_num > 0
}
