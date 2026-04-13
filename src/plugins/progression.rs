// Void Architect — plugins/progression.rs
// EXP system, level-up pause + perk modal, 12 perk implementations,
// Void Core entity, win/lose conditions, Void Shards earn, meta save/load,
// Architect's Sanctum.
// [S0-06, S3-01, S3-02, S3-03, S3-06, S3-07, S3-08, S3-09, S3-10]

use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;
use std::path::PathBuf;

use crate::components::*;
use crate::GameState;

const EXP_SCALE: f32 = 1.4;
const EXP_WAVE_BONUS: u32 = 30;
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
            .insert_resource(LevelUpState::default())
            .insert_resource(RunEndState::default())
            .insert_resource(SanctumState::default())
            .insert_resource(TurretOverloadTimer::default())
            .add_event::<PlayerLeveledUp>()
            .add_event::<ResourceSpent>()
            .add_event::<ResourceGained>()
            .add_event::<RunEndEvent>()
            .add_event::<PerkSelected>()
            .add_event::<SanctumPurchaseEvent>()
            .add_systems(Startup, load_meta_progress)
            .add_systems(OnEnter(GameState::InRun), (reset_run_state, spawn_void_core))
            .add_systems(
                Update,
                (
                    exp_gain_from_kills,
                    exp_wave_bonus,
                    check_level_up,
                    handle_perk_selected,
                    tick_berserker,
                    tick_kill_chain,
                    tick_turret_overload,
                    auto_repair_structures,
                    tick_void_burn,
                    track_stationary_time,
                    monitor_void_core,
                    check_win_condition,
                    check_lose_condition,
                )
                    .run_if(in_state(GameState::InRun)),
            )
            .add_systems(
                Update,
                sanctum_purchase_system.run_if(in_state(GameState::MetaScreen)),
            )
            .add_systems(OnExit(GameState::InRun), (compute_run_end, save_meta_progress));
    }
}

// ---------------------------------------------------------------------------
// Resources & Types
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct LevelUpState {
    pub is_active: bool,
    pub offered: [Option<PerkId>; 3],
    pub hovered: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerkId {
    IronFrame, EfficientBuilder, ChainTurret, AutoRepairDrone,
    TurretOverload, ArchitectsBastion,
    BladeTempo, VoidSense, Berserker, KillChain, VoidResonance, Singularity,
}

impl PerkId {
    pub fn name(self) -> &'static str {
        match self {
            Self::IronFrame        => "Iron Frame",
            Self::EfficientBuilder => "Efficient Builder",
            Self::ChainTurret      => "Chain Turret",
            Self::AutoRepairDrone  => "Auto-Repair Drone",
            Self::TurretOverload   => "Turret Overload",
            Self::ArchitectsBastion=> "Architect's Bastion",
            Self::BladeTempo       => "Blade Tempo",
            Self::VoidSense        => "Void Sense",
            Self::Berserker        => "Berserker",
            Self::KillChain        => "Kill Chain",
            Self::VoidResonance    => "Void Resonance",
            Self::Singularity      => "Singularity",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::IronFrame        => "Wall durability +30%. Structures resist breach.",
            Self::EfficientBuilder => "Construction stone cost -20%.",
            Self::ChainTurret      => "Turret shots chain to 1 additional enemy.",
            Self::AutoRepairDrone  => "Structures self-repair 2 HP/s during night.",
            Self::TurretOverload   => "Turrets fire 2x speed 15s after any structure damage.",
            Self::ArchitectsBastion=> "T2+ structures gain +50 HP void shield on placement.",
            Self::BladeTempo       => "Melee attack speed +25% (cooldown x0.75).",
            Self::VoidSense        => "Immune to Void Crawler EMP. Turrets unaffected.",
            Self::Berserker        => "Below 30% HP: deal 3x damage for 10s (60s cooldown).",
            Self::KillChain        => "First kill each wave: 2x EXP for 10 seconds.",
            Self::VoidResonance    => "Melee hits apply Void Burn: 5 dmg/s for 4s.",
            Self::Singularity      => "Void Explosion pulls all enemies to center first.",
        }
    }
    pub fn tree(self) -> &'static str {
        match self {
            Self::IronFrame | Self::EfficientBuilder | Self::ChainTurret
            | Self::AutoRepairDrone | Self::TurretOverload | Self::ArchitectsBastion => "Builder",
            _ => "Warrior",
        }
    }
    pub const ALL: [PerkId; 12] = [
        Self::IronFrame, Self::EfficientBuilder, Self::ChainTurret,
        Self::AutoRepairDrone, Self::TurretOverload, Self::ArchitectsBastion,
        Self::BladeTempo, Self::VoidSense, Self::Berserker,
        Self::KillChain, Self::VoidResonance, Self::Singularity,
    ];
}

#[derive(Event, Debug, Clone, Copy)]
pub struct PerkSelected { pub perk: PerkId }

#[derive(Resource, Debug, Default)]
pub struct RunEndState {
    pub finished: bool,
    pub victory: bool,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct RunEndEvent { pub victory: bool }

#[derive(Resource, Debug, Default)]
pub struct SanctumState { pub last_message: String }

#[derive(Event, Debug, Clone, Copy)]
pub struct SanctumPurchaseEvent { pub unlock: SanctumUnlock }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanctumUnlock {
    StartingTurret, StartingWalls, VoidAffinity, ColonyBond, UnlockSentinel,
}

impl SanctumUnlock {
    pub fn cost(self) -> u32 {
        match self { Self::StartingTurret=>10, Self::StartingWalls=>8,
            Self::VoidAffinity=>15, Self::ColonyBond=>20, Self::UnlockSentinel=>30 }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::StartingTurret => "Starting Blueprint: Turret (10)",
            Self::StartingWalls  => "Starting Blueprint: Wall Pack x6 (8)",
            Self::VoidAffinity   => "Void Affinity: +50% crystals (15)",
            Self::ColonyBond     => "Colony Bond: +1 starting NPC (20)",
            Self::UnlockSentinel => "Character: The Sentinel (30)",
        }
    }
    pub fn is_owned(self, meta: &MetaProgress) -> bool {
        match self {
            Self::StartingTurret => meta.has_starting_turret,
            Self::StartingWalls  => meta.has_starting_walls,
            Self::VoidAffinity   => meta.void_affinity_active,
            Self::ColonyBond     => meta.colony_bond_active,
            Self::UnlockSentinel => meta.sentinel_unlocked,
        }
    }
}

pub const ALL_SANCTUM_UNLOCKS: [SanctumUnlock; 5] = [
    SanctumUnlock::StartingTurret, SanctumUnlock::StartingWalls,
    SanctumUnlock::VoidAffinity, SanctumUnlock::ColonyBond,
    SanctumUnlock::UnlockSentinel,
];

// ---------------------------------------------------------------------------
// Perk Components (inserted onto Player entity)
// ---------------------------------------------------------------------------

#[derive(Component, Default)] pub struct ActivePerks;
#[derive(Component)] pub struct PerkIronFrame;
#[derive(Component)] pub struct PerkEfficientBuilder;
#[derive(Component)] pub struct PerkChainTurret;
#[derive(Component)] pub struct PerkAutoRepairDrone;
#[derive(Component)] pub struct PerkTurretOverload;
#[derive(Component)] pub struct PerkArchitectsBastion;
#[derive(Component)] pub struct PerkBladeTempo;
#[derive(Component)] pub struct PerkVoidSense;
#[derive(Component)] pub struct PerkBerserker;
#[derive(Component)] pub struct PerkKillChain;
#[derive(Component)] pub struct PerkVoidResonance;
#[derive(Component)] pub struct PerkSingularity;

// ---------------------------------------------------------------------------
// Perk Runtime State Components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Default)]
pub struct BerserkerState {
    pub active_timer: f32,
    pub cooldown_timer: f32,
    pub is_active: bool,
}

#[derive(Component, Debug, Default)]
pub struct KillChainState {
    pub timer: f32,
    pub is_active: bool,
    pub triggered_this_wave: bool,
}

#[derive(Resource, Debug, Default)]
pub struct TurretOverloadTimer { pub remaining: f32 }

#[derive(Component, Debug)]
pub struct VoidBurn { pub remaining: f32 }

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
    if !can_afford(resources, cost) { return false; }
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

pub fn apply_efficient_builder_discount(cost: &mut ResourceCost, has_perk: bool) {
    if has_perk { cost.stone = (cost.stone as f32 * 0.8).ceil() as u32; }
}

pub fn iron_frame_hp_bonus(base_hp: f32, has_perk: bool) -> f32 {
    if has_perk { base_hp * 1.3 } else { base_hp }
}

pub const BASTION_BONUS_HP: f32 = 50.0;

pub fn is_level_up_paused(level_up: &LevelUpState) -> bool {
    level_up.is_active
}

pub fn berserker_damage_mult(state: Option<&BerserkerState>) -> f32 {
    if let Some(s) = state { if s.is_active { return 3.0; } }
    1.0
}

pub fn apply_void_burn(commands: &mut Commands, target: Entity) {
    commands.entity(target).insert(VoidBurn { remaining: 4.0 });
}

pub fn trigger_turret_overload(timer: &mut TurretOverloadTimer) {
    if timer.remaining <= 0.0 {
        timer.remaining = 15.0;
        bevy::log::info!("[TurretOverload] 2x fire rate 15s!");
    }
}

// ---------------------------------------------------------------------------
// Events (kept for backwards compat)
// ---------------------------------------------------------------------------

#[derive(Event, Debug, Clone)]
pub struct ResourceSpent { pub stone: u32, pub scrap: u32, pub void_crystal: u32 }

#[derive(Event, Debug, Clone)]
pub struct ResourceGained { pub stone: u32, pub scrap: u32, pub void_crystal: u32, pub food: u32 }

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn reset_run_state(
    mut resources: ResMut<PlayerResources>,
    mut colony: ResMut<ColonyState>,
    mut run_stats: ResMut<RunStats>,
    mut strategy: ResMut<StrategyTracker>,
    mut level_up: ResMut<LevelUpState>,
    mut run_end: ResMut<RunEndState>,
    mut overload: ResMut<TurretOverloadTimer>,
    meta: Res<MetaProgress>,
) {
    *resources = PlayerResources::default();
    *colony = ColonyState::default();
    *run_stats = RunStats::default();
    *strategy = StrategyTracker::default();
    *level_up = LevelUpState::default();
    *run_end = RunEndState::default();
    *overload = TurretOverloadTimer::default();

    resources.stone = 50;
    resources.scrap = 20;
    resources.food = 10;

    if meta.colony_bond_active { colony.population += 1; }
}

fn spawn_void_core(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.4, 0.0, 0.9),
                custom_size: Some(Vec2::splat(42.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.5),
            ..default()
        },
        VoidCore,
        Health::new(500.0),
    ));
    bevy::log::info!("[VoidCore] Spawned — 500 HP");
}

// S3-01
fn exp_gain_from_kills(
    mut player_q: Query<(&mut Player, Option<&KillChainState>)>,
    mut enemy_died: EventReader<EnemyDied>,
    mut run_stats: ResMut<RunStats>,
    level_up_state: Res<LevelUpState>,
) {
    let Ok((mut player, kc)) = player_q.get_single_mut() else {
        for _ in enemy_died.read() {}
        return;
    };
    let kc_mult: u32 = if kc.map_or(false, |s| s.is_active) { 2 } else { 1 };

    for ev in enemy_died.read() {
        run_stats.total_kills += 1;
        player.exp += ev.exp_reward * kc_mult;
    }
}

fn exp_wave_bonus(
    mut phase_events: EventReader<PhaseChanged>,
    mut player_q: Query<&mut Player, With<crate::plugins::player::PlayerMarker>>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Day || ev.day <= 1 { continue; }
        let Ok(mut p) = player_q.get_single_mut() else { continue };
        p.exp += EXP_WAVE_BONUS;
        bevy::log::info!("[EXP] Wave clear +{} EXP", EXP_WAVE_BONUS);
    }
}

// S3-02
fn check_level_up(
    mut player_q: Query<&mut Player, With<crate::plugins::player::PlayerMarker>>,
    mut level_up_events: EventWriter<PlayerLeveledUp>,
    mut level_up_state: ResMut<LevelUpState>,
) {
    if level_up_state.is_active { return; }
    let Ok(mut player) = player_q.get_single_mut() else { return };

    while player.exp >= player.exp_next && player.level < 50 {
        player.exp -= player.exp_next;
        player.level += 1;
        player.exp_next = (player.exp_next as f32 * EXP_SCALE) as u32;
        level_up_events.send(PlayerLeveledUp { new_level: player.level });

        level_up_state.is_active = true;
        level_up_state.hovered = Some(0);
        level_up_state.offered = pick_three_perks(player.level);
        bevy::log::info!("[Level] Lv.{} | Next: {} EXP", player.level, player.exp_next);
        break; // wait for confirm before next level
    }
}

fn handle_perk_selected(
    mut events: EventReader<PerkSelected>,
    mut commands: Commands,
    mut player_q: Query<
        (Entity, &mut Player),
        With<crate::plugins::player::PlayerMarker>,
    >,
    has_berserker: Query<(), (With<crate::plugins::player::PlayerMarker>, With<BerserkerState>)>,
    has_kc: Query<(), (With<crate::plugins::player::PlayerMarker>, With<KillChainState>)>,
    mut level_up_state: ResMut<LevelUpState>,
) {
    for ev in events.read() {
        let Ok((entity, mut player)) = player_q.get_single_mut() else { continue };
        apply_perk(
            &mut commands, entity, ev.perk, &mut player,
            has_berserker.get(entity).is_ok(),
            has_kc.get(entity).is_ok(),
        );
        level_up_state.is_active = false;
        level_up_state.offered = [None; 3];
        bevy::log::info!("[Perk] Applied: {}", ev.perk.name());
    }
}

// S3-03 — apply_perk
fn apply_perk(
    commands: &mut Commands,
    entity: Entity,
    perk: PerkId,
    player: &mut Player,
    has_berserker: bool,
    has_kc: bool,
) {
    match perk {
        PerkId::IronFrame        => { commands.entity(entity).insert(PerkIronFrame); }
        PerkId::EfficientBuilder => { commands.entity(entity).insert(PerkEfficientBuilder); }
        PerkId::ChainTurret      => { commands.entity(entity).insert(PerkChainTurret); }
        PerkId::AutoRepairDrone  => { commands.entity(entity).insert(PerkAutoRepairDrone); }
        PerkId::TurretOverload   => { commands.entity(entity).insert(PerkTurretOverload); }
        PerkId::ArchitectsBastion=> { commands.entity(entity).insert(PerkArchitectsBastion); }
        PerkId::BladeTempo => {
            commands.entity(entity).insert(PerkBladeTempo);
            player.cooldowns.melee *= 0.75; // +25% attack speed
        }
        PerkId::VoidSense   => { commands.entity(entity).insert(PerkVoidSense); }
        PerkId::Berserker => {
            commands.entity(entity).insert(PerkBerserker);
            if !has_berserker { commands.entity(entity).insert(BerserkerState::default()); }
        }
        PerkId::KillChain => {
            commands.entity(entity).insert(PerkKillChain);
            if !has_kc { commands.entity(entity).insert(KillChainState::default()); }
        }
        PerkId::VoidResonance => { commands.entity(entity).insert(PerkVoidResonance); }
        PerkId::Singularity   => { commands.entity(entity).insert(PerkSingularity); }
    }
}

// S3-03 perk timers
fn tick_berserker(
    mut q: Query<(&Health, &mut BerserkerState), With<PerkBerserker>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let Ok((hp, mut s)) = q.get_single_mut() else { return };
    if s.cooldown_timer > 0.0 { s.cooldown_timer -= dt; }
    if s.is_active {
        s.active_timer -= dt;
        if s.active_timer <= 0.0 {
            s.is_active = false;
            s.cooldown_timer = 60.0;
            bevy::log::info!("[Berserker] Expired — 60s cooldown.");
        }
    } else if s.cooldown_timer <= 0.0 && hp.fraction() < 0.30 {
        s.is_active = true;
        s.active_timer = 10.0;
        bevy::log::info!("[Berserker] ACTIVATED — 3x damage 10s!");
    }
}

fn tick_kill_chain(
    mut q: Query<&mut KillChainState, With<PerkKillChain>>,
    mut phase_events: EventReader<PhaseChanged>,
    mut enemy_died: EventReader<EnemyDied>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let Ok(mut s) = q.get_single_mut() else {
        for _ in phase_events.read() {}
        for _ in enemy_died.read() {}
        return;
    };

    for ev in phase_events.read() {
        if ev.new_phase == Phase::Night {
            s.triggered_this_wave = false;
        }
    }
    for _ in enemy_died.read() {
        if !s.triggered_this_wave && !s.is_active {
            s.is_active = true;
            s.timer = 10.0;
            s.triggered_this_wave = true;
            bevy::log::info!("[KillChain] 2x EXP for 10s!");
        }
    }
    if s.is_active {
        s.timer -= dt;
        if s.timer <= 0.0 { s.is_active = false; }
    }
}

fn tick_turret_overload(mut timer: ResMut<TurretOverloadTimer>, time: Res<Time>) {
    if timer.remaining > 0.0 { timer.remaining -= time.delta_seconds(); }
}

fn auto_repair_structures(
    player_q: Query<(), With<PerkAutoRepairDrone>>,
    mut structure_q: Query<&mut Health, With<Structure>>,
    timer: Res<PhaseTimer>,
    time: Res<Time>,
) {
    if player_q.is_empty() || timer.phase != Phase::Night { return; }
    let heal = 2.0 * time.delta_seconds();
    for mut hp in &mut structure_q { hp.heal(heal); }
}

fn tick_void_burn(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Health, &mut VoidBurn)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (e, mut hp, mut burn) in &mut q {
        hp.damage(5.0 * dt);
        burn.remaining -= dt;
        if burn.remaining <= 0.0 { commands.entity(e).remove::<VoidBurn>(); }
    }
}

// S3-04: stationary time tracking
fn track_stationary_time(
    player_q: Query<&Velocity, With<crate::plugins::player::PlayerMarker>>,
    mut strategy: ResMut<StrategyTracker>,
    timer: Res<PhaseTimer>,
    time: Res<Time>,
) {
    if timer.phase != Phase::Night { return; }
    let dt = time.delta_seconds();
    strategy.total_night_time += dt;
    let Ok(vel) = player_q.get_single() else { return };
    if vel.linvel.length_squared() < 100.0 { strategy.stationary_time += dt; }
}

// S3-06
fn monitor_void_core(
    core_q: Query<&Health, With<VoidCore>>,
    mut events: EventWriter<VoidCoreDamaged>,
    mut last: Local<f32>,
) {
    let Ok(hp) = core_q.get_single() else { return };
    let frac = hp.fraction();
    if (frac - *last).abs() > 0.01 {
        *last = frac;
        events.send(VoidCoreDamaged { remaining_fraction: frac });
        if frac < 0.3 { bevy::log::warn!("[VoidCore] CRITICAL {:.0}%!", frac * 100.0); }
    }
}

// S3-07
fn check_win_condition(
    phase_timer: Res<PhaseTimer>,
    run_end: Res<RunEndState>,
    mut events: EventWriter<RunEndEvent>,
) {
    if run_end.finished { return; }
    if phase_timer.day > 10 && phase_timer.phase == Phase::Day {
        bevy::log::info!("[Win] Survived 10 days — VICTORY!");
        events.send(RunEndEvent { victory: true });
    }
}

fn check_lose_condition(
    player_q: Query<&Health, With<crate::plugins::player::PlayerMarker>>,
    core_q: Query<&Health, With<VoidCore>>,
    run_end: Res<RunEndState>,
    mut events: EventWriter<RunEndEvent>,
) {
    if run_end.finished { return; }
    if player_q.get_single().map_or(false, |h| h.is_dead()) {
        bevy::log::info!("[Lose] Player mati — DEFEAT.");
        events.send(RunEndEvent { victory: false });
        return;
    }
    if core_q.get_single().map_or(false, |h| h.is_dead()) {
        bevy::log::info!("[Lose] Void Core hancur — DEFEAT.");
        events.send(RunEndEvent { victory: false });
    }
}

// S3-08: Compute run end stats, award shards, transition
fn compute_run_end(
    mut run_end_events: EventReader<RunEndEvent>,
    mut run_end_state: ResMut<RunEndState>,
    mut run_stats: ResMut<RunStats>,
    mut meta: ResMut<MetaProgress>,
    phase_timer: Res<PhaseTimer>,
    colony: Res<ColonyState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut last: Option<RunEndEvent> = None;
    for ev in run_end_events.read() { last = Some(*ev); }
    let Some(ev) = last else { return };
    if run_end_state.finished { return; }

    run_end_state.finished = true;
    run_end_state.victory = ev.victory;

    run_stats.days_survived = phase_timer.day.saturating_sub(1).min(10);
    run_stats.npcs_alive = colony.population;

    let shards = calculate_void_shards(&run_stats);
    meta.void_shards += shards;

    bevy::log::info!(
        "[RunEnd] {} | Days:{} Kills:{} Bosses:{} | +{} shards (total:{})",
        if ev.victory { "VICTORY" } else { "DEFEAT" },
        run_stats.days_survived, run_stats.total_kills,
        run_stats.bosses_defeated, shards, meta.void_shards,
    );

    next_state.set(GameState::MetaScreen);
}

// S3-09
fn meta_save_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VoidArchitect")
        .join(META_SAVE_PATH)
}

fn load_meta_progress(mut meta: ResMut<MetaProgress>) {
    let path = meta_save_path();
    if !path.exists() { bevy::log::info!("[Meta] No save, starting fresh."); return; }
    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<MetaProgress>(&json) {
            Ok(loaded) => { *meta = loaded; bevy::log::info!("[Meta] Loaded: {} shards", meta.void_shards); }
            Err(e) => bevy::log::warn!("[Meta] Parse error: {e}"),
        },
        Err(e) => bevy::log::warn!("[Meta] Read error: {e}"),
    }
}

fn save_meta_progress(meta: Res<MetaProgress>) {
    write_meta(&meta);
}

fn write_meta(meta: &MetaProgress) {
    let path = meta_save_path();
    if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
            bevy::log::info!("[Meta] Saved: {} shards", meta.void_shards);
        }
    }
}

// S3-10
fn sanctum_purchase_system(
    mut events: EventReader<SanctumPurchaseEvent>,
    mut meta: ResMut<MetaProgress>,
    mut state: ResMut<SanctumState>,
) {
    for ev in events.read() {
        let u = ev.unlock;
        if u.is_owned(&meta) { state.last_message = format!("{} already owned.", u.label()); continue; }
        if meta.void_shards < u.cost() {
            state.last_message = format!("Need {} shards (have {}).", u.cost(), meta.void_shards);
            continue;
        }
        meta.void_shards -= u.cost();
        match u {
            SanctumUnlock::StartingTurret => meta.has_starting_turret = true,
            SanctumUnlock::StartingWalls  => meta.has_starting_walls = true,
            SanctumUnlock::VoidAffinity   => meta.void_affinity_active = true,
            SanctumUnlock::ColonyBond     => meta.colony_bond_active = true,
            SanctumUnlock::UnlockSentinel => meta.sentinel_unlocked = true,
        }
        write_meta(&meta);
        state.last_message = format!("Unlocked: {} (-{} shards)", u.label(), u.cost());
        bevy::log::info!("[Sanctum] {:?} purchased.", u);
    }
}

// ---------------------------------------------------------------------------
// Simple LCG RNG
// ---------------------------------------------------------------------------

struct SimpleRng { state: u64 }
impl SimpleRng {
    fn new(seed: u64) -> Self { Self { state: seed | 1 } }
    fn next_u32(&mut self) -> u32 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }
}

fn pick_three_perks(level: u32) -> [Option<PerkId>; 3] {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().subsec_nanos() as u64;
    let mut rng = SimpleRng::new(seed ^ (level as u64 * 0xDEAD_BEEF));
    let mut pool: Vec<PerkId> = PerkId::ALL.to_vec();
    let mut result = [None; 3];
    for slot in &mut result {
        if pool.is_empty() { break; }
        let idx = rng.next_u32() as usize % pool.len();
        *slot = Some(pool.remove(idx));
    }
    result
}
