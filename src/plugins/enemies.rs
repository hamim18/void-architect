// Void Architect — plugins/enemies.rs
// Enemy AI behavior + wave spawn system.
// [S1-09] Void Drone — pathfinding sederhana, chase player
// [S1-10] Breacher — target wall terdekat, ignore player
// [S1-11] Rift Stalker — cepat, flanking, prefer NPC target
// [S1-12] Wave spawn system — edge spawning, count scaling, Night trigger
//
// ADR: Semua AI di-handle dalam SATU system (enemy_ai_system) agar tidak ada
// konflik mutable Query<&mut Velocity> antar beberapa system dalam schedule
// yang sama. enemy_attack_system dipisah karena query set-nya berbeda.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::{
    adapt_flags, DamageEvent, Enemy, EnemyType, Health, Npc, PhaseChanged,
    Phase, StrategyTracker, WallMarker,
};
use crate::GameState;
use crate::plugins::player::{Invincible, PlayerMarker};
use crate::plugins::world::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DRONE_SPEED: f32 = 110.0;
const STALKER_SPEED: f32 = 170.0;
const BREACHER_SPEED: f32 = 70.0;

const ENEMY_ATTACK_RADIUS: f32 = 20.0;
const BREACHER_ATTACK_RADIUS: f32 = 28.0;

const STALKER_FLANK_DIST: f32 = 80.0;

const BASE_WAVE_COUNT: u32 = 8;
const WAVE_SCALE_PER_WAVE: u32 = 2;
const SPAWN_EDGE_MARGIN: f32 = 30.0;

const NAV_INTERVAL_DRONE: f32 = 0.25;
const NAV_INTERVAL_BREACHER: f32 = 0.50;
const NAV_INTERVAL_STALKER: f32 = 0.30;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ActiveWave::default())
            .insert_resource(WaveSpawner::default())
            .add_systems(
                Update,
                (
                    handle_wave_spawn_trigger,
                    tick_wave_spawner,
                    enemy_ai_system,
                    enemy_attack_system,
                )
                    .chain()
                    .run_if(in_state(GameState::InRun)),
            );
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Status wave yang sedang berjalan.
#[derive(Resource, Debug, Default)]
pub struct ActiveWave {
    pub wave_num: u32,
    pub is_active: bool,
}

/// Spawner bertahap — spawn satu enemy per interval dari antrian.
/// Tidak spawn semua sekaligus agar tidak lag di frame pertama night.
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

// ---------------------------------------------------------------------------
// AI Components
// ---------------------------------------------------------------------------

/// Kecepatan gerak enemy — di-set saat spawn, dibaca AI system.
#[derive(Component)]
pub struct EnemySpeed(pub f32);

/// State navigasi per enemy — persisten antar frame.
#[derive(Component, Debug, Default)]
pub struct EnemyAiState {
    /// Waypoint tujuan saat ini
    pub waypoint: Option<Vec2>,
    /// Countdown sampai recalculate nav (hemat CPU)
    pub nav_timer: f32,
    /// Fase flank Rift Stalker
    pub flank_phase: FlankPhase,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum FlankPhase {
    #[default]
    Approach, // pendekatan dari sudut
    Strike,   // serangan langsung
}

// ---------------------------------------------------------------------------
// [S1-12] Wave Spawn Trigger — listen PhaseChanged(Night)
// ---------------------------------------------------------------------------

fn handle_wave_spawn_trigger(
    mut phase_events: EventReader<PhaseChanged>,
    mut wave: ResMut<ActiveWave>,
    mut spawner: ResMut<WaveSpawner>,
    strategy: Res<StrategyTracker>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Night {
            continue;
        }

        let wave_num = ev.wave_num;
        wave.wave_num = wave_num;
        wave.is_active = true;

        // Enemy count bertambah 2 tiap wave. Wave 1 = 8, wave 2 = 10, dst.
        let total = BASE_WAVE_COUNT + WAVE_SCALE_PER_WAVE * wave_num.saturating_sub(1);

        // Interval spawn makin pendek di wave lanjut (0.35s → 0.12s minimal)
        let interval = (0.35 - wave_num as f32 * 0.015).max(0.12);

        spawner.queue = build_wave_composition(total, wave_num, &strategy);
        spawner.timer = 0.0;
        spawner.interval = interval;

        bevy::log::info!(
            "[Wave] Night {} — {} enemies akan spawn (interval {:.2}s)",
            wave_num, total, interval
        );
    }
}

/// Tentukan tipe dan posisi tiap enemy dalam wave.
fn build_wave_composition(
    count: u32,
    wave_num: u32,
    strategy: &StrategyTracker,
) -> Vec<EnemySpawnEntry> {
    let mut entries = Vec::with_capacity(count as usize);
    let mut rng = SimpleRng::new(wave_num as u64 * 1337 + 99);

    // Adaptation: wall_reliance > 0.7 → Breacher ratio naik
    let breacher_ratio: f32 = if strategy.wall_reliance() > 0.7 { 0.35 } else { 0.15 };
    // Stalker mulai muncul wave 2+
    let stalker_ratio: f32 = if wave_num >= 2 { 0.20 } else { 0.0 };

    for i in 0..count {
        let roll = rng.next_f32();
        let enemy_type = if roll < breacher_ratio {
            EnemyType::Breacher
        } else if roll < breacher_ratio + stalker_ratio {
            EnemyType::RiftStalker
        } else {
            EnemyType::VoidDrone
        };

        // Distribusi spawn merata ke 4 sisi map
        let side = (i + rng.next_u32() % 2) % 4;
        let pos = spawn_position_on_edge(side, &mut rng);
        entries.push(EnemySpawnEntry { enemy_type, position: pos });
    }

    entries
}

/// Posisi spawn acak di satu sisi edge map.
/// Side: 0=atas, 1=bawah, 2=kiri, 3=kanan.
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

/// Spawn satu enemy dari queue per interval.
fn tick_wave_spawner(
    mut commands: Commands,
    mut spawner: ResMut<WaveSpawner>,
    time: Res<Time>,
) {
    if spawner.queue.is_empty() {
        return;
    }

    spawner.timer -= time.delta_seconds();
    if spawner.timer > 0.0 {
        return;
    }

    if let Some(entry) = spawner.queue.pop() {
        spawn_enemy(&mut commands, entry.enemy_type, entry.position);
        spawner.timer = spawner.interval;
    }
}

/// Spawn satu enemy entity dengan semua component yang diperlukan.
fn spawn_enemy(commands: &mut Commands, enemy_type: EnemyType, pos: Vec2) {
    let (color, size, hp, speed, damage, exp_reward) = enemy_stats(enemy_type);
    let attack_range = if enemy_type == EnemyType::Breacher {
        BREACHER_ATTACK_RADIUS
    } else {
        ENEMY_ATTACK_RADIUS
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
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
        // Rapier — Dynamic + rotation locked agar tidak berputar karena fisika
        RigidBody::Dynamic,
        Collider::ball(size / 2.0 - 2.0),
        LockedAxes::ROTATION_LOCKED,
        Damping {
            linear_damping: 8.0, // friction tinggi → tidak "melayang" saat berhenti
            angular_damping: 1.0,
        },
        Velocity::default(),
        EnemySpeed(speed),
        EnemyAiState::default(),
    ));
}

/// Stat base per enemy type.
/// Returns: (color, size_px, hp, speed, damage, exp_reward)
fn enemy_stats(et: EnemyType) -> (Color, f32, f32, f32, f32, u32) {
    match et {
        EnemyType::VoidDrone   => (Color::srgb(0.85, 0.15, 0.15), 18.0, 40.0, DRONE_SPEED,    5.0, 10),
        EnemyType::RiftStalker => (Color::srgb(0.20, 0.80, 0.20), 14.0, 30.0, STALKER_SPEED,  8.0, 15),
        EnemyType::Breacher    => (Color::srgb(0.55, 0.10, 0.10), 24.0, 60.0, BREACHER_SPEED, 15.0, 20),
        // Tipe post-MVP — stat ada agar tidak panic kalau terspawn via cheat/debug
        EnemyType::VoidCrawler => (Color::srgb(0.20, 0.20, 0.80), 16.0, 25.0, 100.0,  3.0, 12),
        EnemyType::HollowTitan => (Color::srgb(0.60, 0.00, 0.60), 40.0, 180.0, 50.0, 25.0, 60),
        EnemyType::RiftHive    => (Color::srgb(0.80, 0.40, 0.00), 32.0,  50.0,  0.0,  0.0, 30),
        EnemyType::SwarmLord   => (Color::srgb(1.00, 0.00, 0.50), 36.0, 400.0, 60.0, 20.0, 150),
    }
}

/// Adapt flags default per tipe enemy.
fn default_adapt_flags(et: EnemyType) -> u8 {
    match et {
        EnemyType::RiftStalker => adapt_flags::BYPASS_WALL | adapt_flags::TARGET_NPC,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// [S1-09, S1-10, S1-11] Enemy AI System — digabung dalam satu system
//
// Kenapa satu system: kalau drone_ai, breacher_ai, stalker_ai masing-masing
// punya `Query<(&Transform, &mut Velocity, &mut EnemyAiState, &Enemy)>`,
// Bevy tidak bisa membuktikan mereka disjoint → B0001 panic saat runtime.
// Dengan satu system, satu query — dijamin aman.
// ---------------------------------------------------------------------------

fn enemy_ai_system(
    mut enemy_q: Query<(
        &Transform,
        &mut Velocity,
        &mut EnemyAiState,
        &EnemySpeed,
        &Enemy,
    )>,
    player_q: Query<&Transform, With<PlayerMarker>>,
    npc_q: Query<&Transform, With<Npc>>,
    wall_q: Query<&Transform, With<WallMarker>>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Kumpulkan data wall dan NPC sekali, di luar loop enemy
    let wall_positions: Vec<Vec2> = wall_q
        .iter()
        .map(|tf| tf.translation.truncate())
        .collect();

    let npc_positions: Vec<Vec2> = npc_q
        .iter()
        .map(|tf| tf.translation.truncate())
        .collect();

    let dt = time.delta_seconds();

    for (tf, mut vel, mut ai, speed, enemy) in &mut enemy_q {
        let my_pos = tf.translation.truncate();

        match enemy.variant {
            // ----------------------------------------------------------------
            // [S1-09] Void Drone — kejar player, hindari wall
            // ----------------------------------------------------------------
            EnemyType::VoidDrone => {
                ai.nav_timer -= dt;
                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_DRONE;
                    ai.waypoint = Some(drone_waypoint(my_pos, player_pos, &wall_positions));
                }
                let target = ai.waypoint.unwrap_or(player_pos);
                vel.linvel = (target - my_pos).normalize_or_zero() * speed.0;
            }

            // ----------------------------------------------------------------
            // [S1-10] Breacher — kejar wall terdekat, abaikan player
            // ----------------------------------------------------------------
            EnemyType::Breacher => {
                ai.nav_timer -= dt;
                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_BREACHER;
                    // Cari wall terdekat — recalculate tiap 0.5s agar responsif
                    ai.waypoint = wall_positions
                        .iter()
                        .min_by(|&&a, &&b| {
                            a.distance_squared(my_pos)
                                .partial_cmp(&b.distance_squared(my_pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .copied();
                }

                let target = ai.waypoint.unwrap_or(player_pos); // fallback ke player kalau tidak ada wall
                vel.linvel = (target - my_pos).normalize_or_zero() * speed.0;
            }

            // ----------------------------------------------------------------
            // [S1-11] Rift Stalker — prefer NPC, approach dari sudut
            // ----------------------------------------------------------------
            EnemyType::RiftStalker => {
                ai.nav_timer -= dt;

                // Target: NPC terdekat, kalau tidak ada → player
                let primary_target = nearest_vec2(my_pos, &npc_positions)
                    .unwrap_or(player_pos);

                let dist_to_target = my_pos.distance(primary_target);

                if ai.nav_timer <= 0.0 {
                    ai.nav_timer = NAV_INTERVAL_STALKER;

                    if dist_to_target > STALKER_FLANK_DIST * 1.5 {
                        // Masih jauh — approach dari sudut 60°
                        let to_target = (primary_target - my_pos).normalize_or_zero();
                        let angle: f32 = std::f32::consts::FRAC_PI_3; // 60°
                        // Rotate vector to_target sebesar +60°
                        let flank_dir = Vec2::new(
                            to_target.x * angle.cos() - to_target.y * angle.sin(),
                            to_target.x * angle.sin() + to_target.y * angle.cos(),
                        );
                        // Waypoint di sisi target, bukan langsung di target
                        ai.waypoint = Some(primary_target - flank_dir * STALKER_FLANK_DIST);
                        ai.flank_phase = FlankPhase::Approach;
                    } else {
                        // Sudah dekat — langsung ke target
                        ai.waypoint = Some(primary_target);
                        ai.flank_phase = FlankPhase::Strike;
                    }
                }

                let target = ai.waypoint.unwrap_or(primary_target);
                let actual_speed = match ai.flank_phase {
                    FlankPhase::Strike => speed.0 * 1.25, // +25% saat strike
                    FlankPhase::Approach => speed.0,
                };
                vel.linvel = (target - my_pos).normalize_or_zero() * actual_speed;
            }

            // Tipe lain (post-MVP) — diam
            _ => {
                vel.linvel = Vec2::ZERO;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Enemy Attack System
//
// Terpisah dari AI karena query set-nya berbeda: AI butuh &mut Velocity,
// attack butuh &mut Enemy (untuk tick attack_timer) + EventWriter.
// Dengan .chain() di plugin, sistem ini jalan setelah AI — tidak overlap.
// ---------------------------------------------------------------------------

fn enemy_attack_system(
    mut enemy_q: Query<(&Transform, &mut Enemy)>,
    // Player tidak bisa diserang saat invincible (iframe dash)
    player_q: Query<(Entity, &Transform), (With<PlayerMarker>, Without<Invincible>)>,
    npc_q: Query<(Entity, &Transform), With<Npc>>,
    wall_q: Query<(Entity, &Transform), With<WallMarker>>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // Kumpulkan data target sekali — menghindari borrow di dalam loop
    let player_data: Option<(Entity, Vec2)> = player_q
        .get_single()
        .ok()
        .map(|(e, tf)| (e, tf.translation.truncate()));

    let npcs: Vec<(Entity, Vec2)> = npc_q
        .iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .collect();

    let walls: Vec<(Entity, Vec2)> = wall_q
        .iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .collect();

    for (tf, mut enemy) in &mut enemy_q {
        // Tick cooldown
        enemy.attack_timer = (enemy.attack_timer - dt).max(0.0);
        if enemy.attack_timer > 0.0 {
            continue;
        }

        let my_pos = tf.translation.truncate();

        match enemy.variant {
            // Breacher: serang wall terdekat
            EnemyType::Breacher => {
                if let Some(&(wall_e, wall_pos)) = nearest_entity(my_pos, &walls) {
                    if wall_pos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent {
                            target: wall_e,
                            amount: enemy.damage,
                            from_turret: false,
                            from_npc: false,
                        });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }

            // Stalker: prefer NPC, fallback player
            EnemyType::RiftStalker => {
                let npc_hit = nearest_entity(my_pos, &npcs)
                    .filter(|(_, pos)| pos.distance(my_pos) <= enemy.attack_range);

                if let Some(&(npc_e, _)) = npc_hit {
                    damage_events.send(DamageEvent {
                        target: npc_e,
                        amount: enemy.damage,
                        from_turret: false,
                        from_npc: false,
                    });
                    enemy.attack_timer = enemy.attack_cooldown;
                } else if let Some((pe, ppos)) = player_data {
                    if ppos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent {
                            target: pe,
                            amount: enemy.damage,
                            from_turret: false,
                            from_npc: false,
                        });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }

            // Default (Void Drone, dll): serang player
            _ => {
                if let Some((pe, ppos)) = player_data {
                    if ppos.distance(my_pos) <= enemy.attack_range {
                        damage_events.send(DamageEvent {
                            target: pe,
                            amount: enemy.damage,
                            from_turret: false,
                            from_npc: false,
                        });
                        enemy.attack_timer = enemy.attack_cooldown;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pathfinding Helper
// ---------------------------------------------------------------------------

/// Waypoint untuk Void Drone — jalan langsung ke player, hindari wall.
/// Bukan A* penuh: raycast sederhana ke player, kalau ada wall dalam 36px
/// dari jalur, geser waypoint ke samping 52px.
fn drone_waypoint(from: Vec2, to: Vec2, walls: &[Vec2]) -> Vec2 {
    let diff = to - from;
    let dist = diff.length();
    if dist < 1.0 {
        return to;
    }
    let dir = diff / dist;

    // Cari wall yang memblokir jalur langsung
    let blocking = walls.iter().find(|&&wall_pos| {
        let to_wall = wall_pos - from;
        let proj = to_wall.dot(dir);
        // Wall harus berada di antara kita dan target (bukan di belakang/melewati)
        if proj <= 0.0 || proj > dist {
            return false;
        }
        // Jarak tegak lurus dari jalur ke wall — wall dianggap blok kalau < 36px
        (to_wall - dir * proj).length() < 36.0
    });

    if let Some(&wall_pos) = blocking {
        // Geser ke samping untuk menghindari wall
        let perp = Vec2::new(-dir.y, dir.x); // vektor tegak lurus
        // Pilih sisi yang berlawanan dari wall
        let side = if (wall_pos - from).dot(perp) > 0.0 { -1.0 } else { 1.0 };
        from + dir * 48.0 + perp * (side * 52.0)
    } else {
        to // tidak ada halangan
    }
}

// ---------------------------------------------------------------------------
// Utility Helpers
// ---------------------------------------------------------------------------

/// Cari Vec2 terdekat dari slice. Return None kalau kosong.
fn nearest_vec2(from: Vec2, positions: &[Vec2]) -> Option<Vec2> {
    positions
        .iter()
        .min_by(|&&a, &&b| {
            a.distance_squared(from)
                .partial_cmp(&b.distance_squared(from))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// Cari entity terdekat dari slice `(Entity, Vec2)`. Return None kalau kosong.
fn nearest_entity(from: Vec2, entities: &[(Entity, Vec2)]) -> Option<&(Entity, Vec2)> {
    entities.iter().min_by(|(_, a), (_, b)| {
        a.distance_squared(from)
            .partial_cmp(&b.distance_squared(from))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

// ---------------------------------------------------------------------------
// Simple LCG RNG
//
// Tidak pakai crate `rand` agar tidak tambah dependency.
// Reproducible: seed yang sama → spawning pattern yang sama per wave.
// ---------------------------------------------------------------------------

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 } // OR 1 agar tidak pernah 0 (LCG butuh state != 0)
    }

    /// Hasilkan u32 via LCG (konstanta Knuth MMIX).
    fn next_u32(&mut self) -> u32 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    /// Float dalam [0.0, 1.0)
    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    /// Float dalam [min, max)
    fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}
