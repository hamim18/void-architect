// Void Architect — plugins/enemies.rs
// Enemy AI, wave spawn, adaptation tracking (S3-04), Boss 1 Swarm Lord (S3-05).
// [S1-09, S1-10, S1-11, S1-12, S3-04, S3-05]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::{
    adapt_flags, DamageEvent, Enemy, EnemyType, Health, Npc,
    Phase, PhaseChanged, RunStats, StrategyTracker, VoidCore, WallMarker,
};
use crate::GameState;
use crate::plugins::player::{Invincible, PlayerMarker};
// progression imports removed — BerserkerState/RunEndEvent tidak dipakai di enemies.rs
use crate::plugins::world::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DRONE_SPEED: f32 = 110.0;
const STALKER_SPEED: f32 = 170.0;
const BREACHER_SPEED: f32 = 70.0;
const SWARM_LORD_SPEED: f32 = 60.0;

const ENEMY_ATTACK_RADIUS: f32 = 20.0;
const BREACHER_ATTACK_RADIUS: f32 = 28.0;

const BASE_WAVE_COUNT: u32 = 8;
const WAVE_SCALE_PER_WAVE: u32 = 2;
const SPAWN_EDGE_MARGIN: f32 = 30.0;

const NAV_INTERVAL_DRONE: f32 = 0.25;
const NAV_INTERVAL_BREACHER: f32 = 0.50;
const NAV_INTERVAL_STALKER: f32 = 0.30;

// Boss
const SWARM_LORD_HP: f32 = 400.0;
const SWARM_LORD_SPAWN_INTERVAL: f32 = 4.0; // spawn drone tiap 4s
const RIFT_HIVE_HP: f32 = 50.0;
const RIFT_HIVE_SPAWN_INTERVAL: f32 = 10.0; // hive spawn drone tiap 10s
const HIVE_COUNT: usize = 3;

// S3-04 adaptation thresholds (GDD 4.2)
const THRESH_WALL_RELIANCE: f32 = 0.7;
const THRESH_TURRET_KILL: f32 = 0.6;
const THRESH_STATIONARY: f32 = 0.65;
const THRESH_NPC_GUARD: f32 = 0.5;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ActiveWave::default())
            .insert_resource(WaveSpawner::default())
            .insert_resource(BossState::default())
            .add_systems(
                Update,
                (
                    handle_wave_spawn_trigger,
                    tick_wave_spawner,
                    enemy_ai_system,
                    enemy_attack_system,
                    // S3-04
                    update_strategy_tracker,
                    // S3-05
                    swarm_lord_ai,
                    rift_hive_spawn,
                    check_boss_phase_transition,
                    check_boss_death,
                )
                    .run_if(in_state(GameState::InRun)),
            );
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct ActiveWave {
    pub wave_num: u32,
    pub is_active: bool,
    pub enemies_remaining: u32,
}

#[derive(Resource, Debug, Default)]
struct WaveSpawner {
    queue: Vec<EnemySpawnEntry>,
    timer: f32,
    interval: f32,
}

#[derive(Debug, Clone)]
struct EnemySpawnEntry {
    enemy_type: EnemyType,
    position: Vec2,
}

/// S3-05: Global boss state.
#[derive(Resource, Debug, Default)]
pub struct BossState {
    pub boss_active: bool,
    pub boss_entity: Option<Entity>,
    pub hives: Vec<Entity>,
    pub phase: u8,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct EnemySpeed(pub f32);

#[derive(Component, Debug, Default)]
pub struct EnemyAiState {
    pub waypoint: Option<Vec2>,
    pub nav_timer: f32,
    pub flank_phase: FlankPhase,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum FlankPhase { #[default] Approach, Strike }

/// S3-05: Marker untuk Swarm Lord boss.
#[derive(Component, Debug)]
pub struct SwarmLord {
    pub spawn_timer: f32,
    pub phase: u8,            // 1 atau 2
    pub hives_alive: u8,      // berapa Rift Hive yang masih ada
    pub immune: bool,         // immune sampai semua hive destroyed
}

/// S3-05: Rift Hive spawner (boss mechanic).
#[derive(Component, Debug)]
pub struct RiftHive {
    pub spawn_timer: f32,
}

// ---------------------------------------------------------------------------
// S1-12: Wave Spawn Trigger
// ---------------------------------------------------------------------------

fn handle_wave_spawn_trigger(
    mut phase_events: EventReader<PhaseChanged>,
    mut wave: ResMut<ActiveWave>,
    mut spawner: ResMut<WaveSpawner>,
    mut boss_state: ResMut<BossState>,
    strategy: Res<StrategyTracker>,
    mut commands: Commands,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Night { continue; }

        let wave_num = ev.wave_num;
        wave.wave_num = wave_num;
        wave.is_active = true;

        // S3-05: Boss wave setiap kelipatan 5
        if wave_num % 5 == 0 && wave_num > 0 {
            spawn_swarm_lord(&mut commands, &mut boss_state);
            bevy::log::info!("[Boss] SWARM LORD spawns at wave {}!", wave_num);
            // Tetap spawn beberapa drone sebagai pendamping
            let companion_count = 4u32;
            let interval = 0.5;
            spawner.queue = build_drone_companions(companion_count);
            spawner.timer = 2.0; // delay sebelum companion spawn
            spawner.interval = interval;
        } else {
            let total = BASE_WAVE_COUNT + WAVE_SCALE_PER_WAVE * wave_num.saturating_sub(1);
            let interval = (0.35 - wave_num as f32 * 0.015).max(0.12);
            spawner.queue = build_wave_composition(total, wave_num, &strategy);
            spawner.timer = 0.0;
            spawner.interval = interval;
            bevy::log::info!("[Wave] Night {} — {} enemies (interval {:.2}s)", wave_num, total, interval);
        }
    }
}

// ---------------------------------------------------------------------------
// S3-04: Update adaptation ratios tiap akhir malam
// ---------------------------------------------------------------------------

fn update_strategy_tracker(
    mut phase_events: EventReader<PhaseChanged>,
    strategy: Res<StrategyTracker>,
) {
    for ev in phase_events.read() {
        // Hitung ulang rasio saat Day dimulai (akhir malam)
        if ev.new_phase == Phase::Day {
            if strategy.structure_damage_total > 0.0 {
                let wall_ratio = strategy.wall_damage_total / strategy.structure_damage_total;
                // Log adaptation status
                if wall_ratio > THRESH_WALL_RELIANCE {
                    bevy::log::info!("[Adapt] Wall reliance {:.0}% → Breacher counter aktif", wall_ratio * 100.0);
                }
            }
            if strategy.total_kills > 0 {
                let turret_ratio = strategy.turret_kills as f32 / strategy.total_kills as f32;
                if turret_ratio > THRESH_TURRET_KILL {
                    bevy::log::info!("[Adapt] Turret kills {:.0}% → VoidCrawler akan muncul", turret_ratio * 100.0);
                }
            }
        }
    }
}

/// Build wave dengan adaptation flags dari StrategyTracker.
fn build_wave_composition(count: u32, wave_num: u32, strategy: &StrategyTracker) -> Vec<EnemySpawnEntry> {
    let mut entries = Vec::with_capacity(count as usize);
    let mut rng = SimpleRng::new(wave_num as u64 * 1337 + 99);

    // S3-04: Adaptation ratios drive composition
    let wall_reliance = strategy.wall_reliance();
    let turret_ratio = strategy.turret_kill_ratio();
    let npc_ratio = strategy.npc_kill_ratio();

    // Breacher ratio naik kalau player terlalu andalkan wall
    let breacher_ratio: f32 = if wall_reliance > THRESH_WALL_RELIANCE { 0.35 } else { 0.15 };
    // Stalker mulai wave 2, lebih banyak kalau NPC kill ratio tinggi
    let stalker_ratio: f32 = if wave_num >= 2 {
        if npc_ratio > THRESH_NPC_GUARD { 0.30 } else { 0.20 }
    } else { 0.0 };

    for i in 0..count {
        let roll = rng.next_f32();
        let mut enemy_type = if roll < breacher_ratio {
            EnemyType::Breacher
        } else if roll < breacher_ratio + stalker_ratio {
            EnemyType::RiftStalker
        } else {
            EnemyType::VoidDrone
        };

        // S3-04: Inject VoidCrawler kalau turret kills tinggi (wave 3+)
        if wave_num >= 3 && turret_ratio > THRESH_TURRET_KILL && i % 5 == 0 {
            enemy_type = EnemyType::VoidCrawler;
        }

        let side = (i + rng.next_u32() % 2) % 4;
        let pos = spawn_position_on_edge(side, &mut rng);
        entries.push(EnemySpawnEntry { enemy_type, position: pos });
    }
    entries
}

fn build_drone_companions(count: u32) -> Vec<EnemySpawnEntry> {
    let mut rng = SimpleRng::new(0xBEEF_DEAD);
    (0..count).map(|i| {
        let pos = spawn_position_on_edge(i % 4, &mut rng);
        EnemySpawnEntry { enemy_type: EnemyType::VoidDrone, position: pos }
    }).collect()
}

fn spawn_position_on_edge(side: u32, rng: &mut SimpleRng) -> Vec2 {
    let half_w = (MAP_WIDTH as f32 * TILE_SIZE) / 2.0;
    let half_h = (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0;
    let m = SPAWN_EDGE_MARGIN;
    match side % 4 {
        0 => Vec2::new(rng.next_f32_range(-half_w + m, half_w - m), half_h - m),
        1 => Vec2::new(rng.next_f32_range(-half_w + m, half_w - m), -half_h + m),
        2 => Vec2::new(-half_w + m, rng.next_f32_range(-half_h + m, half_h - m)),
        _ => Vec2::new(half_w - m, rng.next_f32_range(-half_h + m, half_h - m)),
    }
}

fn tick_wave_spawner(mut commands: Commands, mut spawner: ResMut<WaveSpawner>, time: Res<Time>) {
    if spawner.queue.is_empty() { return; }
    spawner.timer -= time.delta_seconds();
    if spawner.timer > 0.0 { return; }
    if let Some(entry) = spawner.queue.pop() {
        spawn_enemy(&mut commands, entry.enemy_type, entry.position);
        spawner.timer = spawner.interval;
    }
}

fn spawn_enemy(commands: &mut Commands, enemy_type: EnemyType, pos: Vec2) {
    let (color, size, hp, speed, damage, exp_reward) = enemy_stats(enemy_type);
    let attack_range = if enemy_type == EnemyType::Breacher { BREACHER_ATTACK_RADIUS } else { ENEMY_ATTACK_RADIUS };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color, custom_size: Some(Vec2::splat(size)), ..default() },
            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
            ..default()
        },
        Enemy {
            variant: enemy_type,
            adapt_flags: default_adapt_flags(enemy_type),
            damage,
            attack_range,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
            exp_reward,
        },
        Health::new(hp),
        RigidBody::Dynamic,
        Collider::ball(size / 2.0 - 2.0),
        LockedAxes::ROTATION_LOCKED,
        Damping { linear_damping: 8.0, angular_damping: 1.0 },
        Velocity::default(),
        EnemySpeed(speed),
        EnemyAiState::default(),
    ));
}

fn enemy_stats(et: EnemyType) -> (Color, f32, f32, f32, f32, u32) {
    match et {
        EnemyType::VoidDrone   => (Color::srgb(0.85, 0.15, 0.15), 18.0, 40.0,  DRONE_SPEED,   5.0,  10),
        EnemyType::RiftStalker => (Color::srgb(0.20, 0.80, 0.20), 14.0, 30.0,  STALKER_SPEED, 8.0,  15),
        EnemyType::Breacher    => (Color::srgb(0.55, 0.10, 0.10), 24.0, 60.0,  BREACHER_SPEED,15.0, 20),
        EnemyType::VoidCrawler => (Color::srgb(0.20, 0.20, 0.80), 16.0, 25.0,  100.0,          3.0,  12),
        EnemyType::HollowTitan => (Color::srgb(0.60, 0.00, 0.60), 40.0, 180.0, 50.0,          25.0, 60),
        EnemyType::RiftHive    => (Color::srgb(0.80, 0.40, 0.00), 32.0, 50.0,  0.0,            0.0,  30),
        EnemyType::SwarmLord   => (Color::srgb(1.00, 0.00, 0.50), 40.0, SWARM_LORD_HP, SWARM_LORD_SPEED, 20.0, 150),
    }
}

fn default_adapt_flags(et: EnemyType) -> u8 {
    match et {
        EnemyType::RiftStalker => adapt_flags::BYPASS_WALL | adapt_flags::TARGET_NPC,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// S3-05: Swarm Lord Boss
// ---------------------------------------------------------------------------

fn spawn_swarm_lord(commands: &mut Commands, boss_state: &mut BossState) {
    // Spawn boss di atas map
    let boss = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.5),
                custom_size: Some(Vec2::splat(44.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 280.0, 2.0),
            ..default()
        },
        Enemy {
            variant: EnemyType::SwarmLord,
            adapt_flags: 0,
            damage: 20.0,
            attack_range: 30.0,
            attack_cooldown: 2.0,
            attack_timer: 0.0,
            exp_reward: 150,
        },
        Health::new(SWARM_LORD_HP),
        SwarmLord {
            spawn_timer: SWARM_LORD_SPAWN_INTERVAL,
            phase: 1,
            hives_alive: HIVE_COUNT as u8,
            immune: true,
        },
        RigidBody::Dynamic,
        Collider::ball(20.0),
        LockedAxes::ROTATION_LOCKED,
        Damping { linear_damping: 5.0, angular_damping: 1.0 },
        Velocity::default(),
        EnemySpeed(SWARM_LORD_SPEED),
        EnemyAiState::default(),
    )).id();

    // Spawn 3 Rift Hive di posisi berbeda
    let hive_positions = [
        Vec2::new(-200.0, 150.0),
        Vec2::new(200.0, 150.0),
        Vec2::new(0.0, -200.0),
    ];

    let mut hives = Vec::new();
    for &hive_pos in &hive_positions {
        let hive = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.9, 0.4, 0.0),
                    custom_size: Some(Vec2::splat(32.0)),
                    ..default()
                },
                transform: Transform::from_xyz(hive_pos.x, hive_pos.y, 1.5),
                ..default()
            },
            Enemy {
                variant: EnemyType::RiftHive,
                adapt_flags: 0,
                damage: 0.0,
                attack_range: 0.0,
                attack_cooldown: 999.0,
                attack_timer: 0.0,
                exp_reward: 30,
            },
            Health::new(RIFT_HIVE_HP),
            RiftHive { spawn_timer: RIFT_HIVE_SPAWN_INTERVAL },
            RigidBody::Fixed,
            Collider::ball(14.0),
            Velocity::default(),
            EnemyAiState::default(),
            EnemySpeed(0.0),
        )).id();
        hives.push(hive);
    }

    boss_state.boss_active = true;
    boss_state.boss_entity = Some(boss);
    boss_state.hives = hives;
    boss_state.phase = 1;

    bevy::log::info!("[Boss] Swarm Lord + {} Rift Hives spawned. Boss IMMUNE until all hives destroyed.", HIVE_COUNT);
}

/// Swarm Lord AI: gerak ke Void Core, spawn drone secara berkala.
fn swarm_lord_ai(
    mut commands: Commands,
    mut boss_q: Query<(&Transform, &mut Velocity, &mut SwarmLord, &EnemySpeed)>,
    core_q: Query<&Transform, With<VoidCore>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let core_pos = core_q.get_single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);

    let Ok((tf, mut vel, mut swarm, speed)) = boss_q.get_single_mut() else { return };
    let my_pos = tf.translation.truncate();

    // Gerak ke Void Core
    let dir = (core_pos - my_pos).normalize_or_zero();
    vel.linvel = dir * speed.0;

    // Spawn drone secara berkala
    swarm.spawn_timer -= dt;
    if swarm.spawn_timer <= 0.0 {
        swarm.spawn_timer = SWARM_LORD_SPAWN_INTERVAL;
        let mut rng = SimpleRng::new(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().subsec_nanos() as u64);
        // Spawn drone di sekitar boss
        let offset = Vec2::new(
            rng.next_f32_range(-60.0, 60.0),
            rng.next_f32_range(-60.0, 60.0),
        );
        spawn_enemy(&mut commands, EnemyType::VoidDrone, my_pos + offset);

        let enemy_type = if swarm.phase == 2 { EnemyType::RiftStalker } else { EnemyType::VoidDrone };
        bevy::log::debug!("[Boss] SwarmLord spawns {:?}", enemy_type);
    }
}

/// Rift Hive: diam, spawn drone tiap interval.
fn rift_hive_spawn(
    mut commands: Commands,
    mut hive_q: Query<(&Transform, &mut RiftHive)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (tf, mut hive) in &mut hive_q {
        hive.spawn_timer -= dt;
        if hive.spawn_timer <= 0.0 {
            hive.spawn_timer = RIFT_HIVE_SPAWN_INTERVAL;
            let pos = tf.translation.truncate();
            // Spawn drone di sekitar hive
            let mut rng = SimpleRng::new(pos.x as u64 ^ pos.y as u64);
            let offset = Vec2::new(rng.next_f32_range(-40.0, 40.0), rng.next_f32_range(-40.0, 40.0));
            spawn_enemy(&mut commands, EnemyType::VoidDrone, pos + offset);
        }
    }
}

/// Cek apakah Rift Hive mati → update SwarmLord immunity.
fn check_boss_phase_transition(
    hive_q: Query<&Health, With<RiftHive>>,
    mut boss_q: Query<(&Health, &mut SwarmLord)>,
    mut boss_state: ResMut<BossState>,
) {
    let Ok((boss_hp, mut swarm)) = boss_q.get_single_mut() else { return };

    // Hitung hive yang masih hidup
    let hives_alive = hive_q.iter().filter(|h| !h.is_dead()).count() as u8;
    swarm.hives_alive = hives_alive;

    // Boss tidak bisa damage saat ada hive hidup
    swarm.immune = hives_alive > 0;

    // Phase 2: boss HP < 50%
    if swarm.phase == 1 && boss_hp.fraction() < 0.5 && !swarm.immune {
        swarm.phase = 2;
        boss_state.phase = 2;
        bevy::log::info!("[Boss] Swarm Lord PHASE 2 — faster spawns, Stalkers!");
    }
}

/// Cek kematian boss.
fn check_boss_death(
    boss_q: Query<(&Health, &Transform), With<SwarmLord>>,
    mut boss_state: ResMut<BossState>,
    mut run_stats: ResMut<RunStats>,
) {
    let Ok((hp, _tf)) = boss_q.get_single() else { return };
    if !hp.is_dead() { return; }

    run_stats.bosses_defeated += 1;
    boss_state.boss_active = false;
    boss_state.boss_entity = None;

    bevy::log::info!("[Boss] SWARM LORD DEFEATED! Total bosses: {}", run_stats.bosses_defeated);
}

// ---------------------------------------------------------------------------
// S1-09, S1-10, S1-11: Enemy AI (single system — ADR-15)
// ---------------------------------------------------------------------------

fn enemy_ai_system(
    mut enemy_q: Query<(&Transform, &mut Velocity, &mut EnemyAiState, &EnemySpeed, &Enemy),
        Without<SwarmLord>>,
    player_q: Query<&Transform, With<PlayerMarker>>,
    npc_q: Query<&Transform, With<Npc>>,
    wall_q: Query<&Transform, With<WallMarker>>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation.truncate();
    let wall_positions: Vec<Vec2> = wall_q.iter().map(|t| t.translation.truncate()).collect();
    let npc_positions: Vec<Vec2> = npc_q.iter().map(|t| t.translation.truncate()).collect();
    let dt = time.delta_seconds();

    for (tf, mut vel, mut ai, speed, enemy) in &mut enemy_q {
        let my_pos = tf.translation.truncate();
        match enemy.variant {
            EnemyType::VoidDrone | EnemyType::VoidCrawler => {
                ai.nav_timer -= dt;
                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_DRONE;
                    ai.waypoint = Some(drone_waypoint(my_pos, player_pos, &wall_positions));
                }
                let target = ai.waypoint.unwrap_or(player_pos);
                vel.linvel = (target - my_pos).normalize_or_zero() * speed.0;
            }
            EnemyType::Breacher => {
                ai.nav_timer -= dt;
                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_BREACHER;
                    ai.waypoint = wall_positions.iter()
                        .min_by(|&&a, &&b| a.distance_squared(my_pos)
                            .partial_cmp(&b.distance_squared(my_pos))
                            .unwrap_or(std::cmp::Ordering::Equal))
                        .copied();
                }
                let target = ai.waypoint.unwrap_or(player_pos);
                vel.linvel = (target - my_pos).normalize_or_zero() * speed.0;
            }
            EnemyType::RiftStalker => {
                ai.nav_timer -= dt;
                // Target NPC terdekat atau player
                let primary = nearest_vec2(my_pos, &npc_positions).unwrap_or(player_pos);
                let dist = my_pos.distance(primary);
                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_STALKER;
                    if dist > 120.0 {
                        let to = (primary - my_pos).normalize_or_zero();
                        let angle: f32 = std::f32::consts::FRAC_PI_3;
                        let flank = Vec2::new(
                            to.x * angle.cos() - to.y * angle.sin(),
                            to.x * angle.sin() + to.y * angle.cos(),
                        );
                        ai.waypoint = Some(primary - flank * 80.0);
                        ai.flank_phase = FlankPhase::Approach;
                    } else {
                        ai.waypoint = Some(primary);
                        ai.flank_phase = FlankPhase::Strike;
                    }
                }
                let actual_speed = if ai.flank_phase == FlankPhase::Strike { speed.0 * 1.25 } else { speed.0 };
                let target = ai.waypoint.unwrap_or(primary);
                vel.linvel = (target - my_pos).normalize_or_zero() * actual_speed;
            }
            _ => { vel.linvel = Vec2::ZERO; }
        }
    }
}

fn enemy_attack_system(
    mut enemy_q: Query<(&Transform, &mut Enemy), Without<RiftHive>>,
    player_q: Query<(Entity, &Transform), (With<PlayerMarker>, Without<Invincible>)>,
    npc_q: Query<(Entity, &Transform), With<Npc>>,
    wall_q: Query<(Entity, &Transform), With<WallMarker>>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let player_data: Option<(Entity, Vec2)> = player_q.get_single().ok()
        .map(|(e, tf)| (e, tf.translation.truncate()));
    let npcs: Vec<(Entity, Vec2)> = npc_q.iter().map(|(e, tf)| (e, tf.translation.truncate())).collect();
    let walls: Vec<(Entity, Vec2)> = wall_q.iter().map(|(e, tf)| (e, tf.translation.truncate())).collect();

    for (tf, mut enemy) in &mut enemy_q {
        enemy.attack_timer = (enemy.attack_timer - dt).max(0.0);
        if enemy.attack_timer > 0.0 { continue; }
        let my_pos = tf.translation.truncate();

        match enemy.variant {
            EnemyType::Breacher => {
                if let Some(&(wall_e, wall_pos)) = nearest_entity(my_pos, &walls) {
                    if wall_pos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent { target: wall_e, amount: enemy.damage, from_turret: false, from_npc: false });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }
            EnemyType::RiftStalker => {
                let npc_hit = nearest_entity(my_pos, &npcs)
                    .filter(|(_, p)| p.distance(my_pos) <= enemy.attack_range);
                if let Some(&(npc_e, _)) = npc_hit {
                    damage_events.send(DamageEvent { target: npc_e, amount: enemy.damage, from_turret: false, from_npc: false });
                    enemy.attack_timer = enemy.attack_cooldown;
                } else if let Some((pe, ppos)) = player_data {
                    if ppos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent { target: pe, amount: enemy.damage, from_turret: false, from_npc: false });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }
            _ => {
                if let Some((pe, ppos)) = player_data {
                    if ppos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent { target: pe, amount: enemy.damage, from_turret: false, from_npc: false });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pathfinding helpers
// ---------------------------------------------------------------------------

fn drone_waypoint(from: Vec2, to: Vec2, walls: &[Vec2]) -> Vec2 {
    let diff = to - from;
    let dist = diff.length();
    if dist < 1.0 { return to; }
    let dir = diff / dist;
    let blocking = walls.iter().find(|&&wall_pos| {
        let to_wall = wall_pos - from;
        let proj = to_wall.dot(dir);
        if proj <= 0.0 || proj > dist { return false; }
        (to_wall - dir * proj).length() < 36.0
    });
    if let Some(&wall_pos) = blocking {
        let perp = Vec2::new(-dir.y, dir.x);
        let side = if (wall_pos - from).dot(perp) > 0.0 { -1.0 } else { 1.0 };
        from + dir * 48.0 + perp * (side * 52.0)
    } else { to }
}

fn nearest_vec2(from: Vec2, positions: &[Vec2]) -> Option<Vec2> {
    positions.iter()
        .min_by(|&&a, &&b| a.distance_squared(from).partial_cmp(&b.distance_squared(from))
            .unwrap_or(std::cmp::Ordering::Equal))
        .copied()
}

fn nearest_entity(from: Vec2, entities: &[(Entity, Vec2)]) -> Option<&(Entity, Vec2)> {
    entities.iter().min_by(|(_, a), (_, b)|
        a.distance_squared(from).partial_cmp(&b.distance_squared(from))
            .unwrap_or(std::cmp::Ordering::Equal))
}

// ---------------------------------------------------------------------------
// SimpleRng
// ---------------------------------------------------------------------------

struct SimpleRng { state: u64 }
impl SimpleRng {
    fn new(seed: u64) -> Self { Self { state: seed | 1 } }
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }
    fn next_f32(&mut self) -> f32 { self.next_u32() as f32 / u32::MAX as f32 }
    fn next_f32_range(&mut self, min: f32, max: f32) -> f32 { min + self.next_f32() * (max - min) }
}
