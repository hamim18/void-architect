// Void Architect — plugins/progression.rs
// Resource system, EXP tracking, meta-progression save/load.
// [S0-06, S3-01, S3-08, S3-09, S3-10]

use bevy::prelude::*;
use std::path::PathBuf;

use crate::components::*;
use crate::GameState;

const EXP_SCALE: f32 = 1.4;
const META_SAVE_PATH: &str = "void_architect_meta.json";

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerResources::default())
            .insert_resource(ColonyState::default())
            .insert_resource(RunStats::default())
            .insert_resource(MetaProgress::default())
            .insert_resource(StrategyTracker::default())
            .add_event::<PlayerLeveledUp>()
            .add_event::<ResourceSpent>()
            .add_event::<ResourceGained>()
            .add_systems(Startup, load_meta_progress)
            .add_systems(OnEnter(GameState::InRun), reset_run_state)
            .add_systems(Update, (
                exp_gain_from_kills,
                check_level_up,
            ).run_if(in_state(GameState::InRun)))
            .add_systems(OnExit(GameState::InRun), save_meta_progress);
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Event, Debug, Clone)]
pub struct ResourceSpent {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
}

#[derive(Event, Debug, Clone)]
pub struct ResourceGained {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
    pub food: u32,
}

// ---------------------------------------------------------------------------
// Resource API (pub — dipakai structures.rs, dll)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceCost {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
}

pub fn can_afford(resources: &PlayerResources, cost: &ResourceCost) -> bool {
    resources.stone >= cost.stone
        && resources.scrap >= cost.scrap
        && resources.void_crystal >= cost.void_crystal
}

pub fn spend_resources(resources: &mut PlayerResources, cost: &ResourceCost) -> bool {
    if !can_afford(resources, cost) {
        return false;
    }
    resources.stone -= cost.stone;
    resources.scrap -= cost.scrap;
    resources.void_crystal -= cost.void_crystal;
    true
}

pub fn get_crystal_multiplier(meta: &MetaProgress) -> f32 {
    if meta.void_affinity_active { 1.5 } else { 1.0 }
}

pub fn calculate_void_shards(stats: &RunStats) -> u32 {
    stats.days_survived + stats.bosses_defeated * 5
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn exp_gain_from_kills(
    mut player_q: Query<&mut Player, With<crate::plugins::player::PlayerMarker>>,
    mut enemy_died_events: EventReader<EnemyDied>,
) {
    let Ok(mut player) = player_q.get_single_mut() else { return };
    for ev in enemy_died_events.read() {
        player.exp += ev.exp_reward;
    }
}

fn check_level_up(
    mut player_q: Query<&mut Player, With<crate::plugins::player::PlayerMarker>>,
    mut level_up_events: EventWriter<PlayerLeveledUp>,
) {
    let Ok(mut player) = player_q.get_single_mut() else { return };
    while player.exp >= player.exp_next && player.level < 50 {
        player.exp -= player.exp_next;
        player.level += 1;
        player.exp_next = (player.exp_next as f32 * EXP_SCALE) as u32;
        level_up_events.send(PlayerLeveledUp { new_level: player.level });
    }
}

fn reset_run_state(
    mut resources: ResMut<PlayerResources>,
    mut colony: ResMut<ColonyState>,
    mut run_stats: ResMut<RunStats>,
    mut strategy: ResMut<StrategyTracker>,
    meta: Res<MetaProgress>,
) {
    *resources = PlayerResources::default();
    *colony = ColonyState::default();
    *run_stats = RunStats::default();
    *strategy = StrategyTracker::default();

    resources.stone = 50;
    resources.scrap = 20;

    if meta.colony_bond_active {
        colony.population += 1;
    }
}

// ---------------------------------------------------------------------------
// Meta Save / Load
// ---------------------------------------------------------------------------

fn meta_save_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VoidArchitect")
        .join(META_SAVE_PATH)
}

fn load_meta_progress(mut meta: ResMut<MetaProgress>) {
    let path = meta_save_path();
    if !path.exists() {
        bevy::log::info!("[Meta] No save found, starting fresh.");
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<MetaProgress>(&json) {
            Ok(loaded) => {
                *meta = loaded;
                bevy::log::info!("[Meta] Loaded: {} void shards", meta.void_shards);
            }
            Err(e) => bevy::log::warn!("[Meta] Parse error: {e}. Starting fresh."),
        },
        Err(e) => bevy::log::warn!("[Meta] Read error: {e}. Starting fresh."),
    }
}

fn save_meta_progress(meta: Res<MetaProgress>) {
    let path = meta_save_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&*meta) {
        Ok(json) => {
            let tmp = path.with_extension("tmp");
            if std::fs::write(&tmp, &json).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
                bevy::log::info!("[Meta] Saved: {} void shards", meta.void_shards);
            } else {
                bevy::log::error!("[Meta] Failed to write save file.");
            }
        }
        Err(e) => bevy::log::error!("[Meta] Serialize error: {e}"),
    }
}
