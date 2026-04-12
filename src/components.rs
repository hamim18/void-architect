// Void Architect — components.rs
// Core ECS components. Didefinisikan di satu tempat untuk menghindari
// refactor besar-besaran antar plugin. Semua plugin import dari sini.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Spatial & Physics
// ---------------------------------------------------------------------------

/// 2D world position — dipakai sebagai shortcut reference di luar Transform.
/// Untuk physics sebenarnya, bevy_rapier2d menggunakan Transform langsung.
#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Current velocity vector (unit/s). Rapier2d yang menggerakkan rigidbody,
/// tapi komponen ini dipakai sistem lain yang butuh baca kecepatan.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec2);

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

/// Health pool untuk player, enemy, struktur, Void Core, dan NPC.
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

/// Data spesifik player yang tidak masuk ke Progression.
#[derive(Component, Debug, Clone)]
pub struct Player {
    pub level: u32,
    pub exp: u32,
    pub exp_next: u32, // threshold next level-up
    /// Aura radius yang mempengaruhi morale NPC di sekitarnya (px).
    pub morale_aura: f32,
    /// Arah hadap (radian) berdasarkan mouse position.
    pub facing: f32,
    /// Flag apakah sedang dalam state dodge dash (untuk iframes).
    pub is_dashing: bool,
    /// Timer cooldown tiap ability dalam detik.
    pub cooldowns: AbilityCooldowns,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            level: 1,
            exp: 0,
            exp_next: 500,
            morale_aura: 150.0,
            facing: 0.0,
            is_dashing: false,
            cooldowns: AbilityCooldowns::default(),
        }
    }
}

/// Cooldown tracker untuk semua ability player (dalam detik tersisa).
#[derive(Debug, Clone, Default)]
pub struct AbilityCooldowns {
    pub melee: f32,        // 0.4s
    pub dash: f32,         // 3.0s
    pub grenade: f32,      // 12.0s
    pub void_explosion: f32, // 45.0s
    pub repair_pulse: f32, // 20.0s
}

// ---------------------------------------------------------------------------
// Enemy
// ---------------------------------------------------------------------------

/// Tipe enemy — menentukan AI behavior dan stat base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    VoidDrone,
    RiftStalker,
    Breacher,
    VoidCrawler,
    HollowTitan,
    RiftHive,
    SwarmLord, // Boss
}

/// Komponen utama enemy entity.
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub variant: EnemyType,
    /// Flag modifikasi behavior dari adaptation system.
    pub adapt_flags: u8,
    /// Base damage per serangan.
    pub damage: f32,
    /// Jarak serangan (px).
    pub attack_range: f32,
    /// Cooldown serangan (detik).
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    /// EXP yang diberikan saat mati.
    pub exp_reward: u32,
}

// Adapt flags sebagai konstanta bit
pub mod adapt_flags {
    pub const TARGET_NPC: u8       = 0b0000_0001;
    pub const BYPASS_WALL: u8      = 0b0000_0010;
    pub const ENRAGED: u8          = 0b0000_0100;
}

// ---------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------

/// Tipe struktur — menentukan behavior dan visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureType {
    Wall,
    SpikeTrap,
    Farm,
    House,
    Turret,
    SlowField,
    Well,
    Barracks,
}

/// Komponen utama struktur yang di-place player.
#[derive(Component, Debug, Clone)]
pub struct Structure {
    pub tier: u8,
    pub structure_type: StructureType,
    /// Durability 0.0–1.0 (fraction of max HP).
    pub durability: f32,
    /// Grid position dalam tile units.
    pub grid_pos: IVec2,
}

/// Marker untuk entitas Wall — dicari oleh Breacher AI.
#[derive(Component)]
pub struct WallMarker;

/// Marker untuk entitas Turret — dikenai EMP Void Crawler.
#[derive(Component)]
pub struct TurretMarker;

/// Marker untuk entitas Farm.
#[derive(Component)]
pub struct FarmMarker;

/// Marker untuk entitas House.
#[derive(Component)]
pub struct HouseMarker;

/// State apakah turret sedang di-disable oleh EMP.
#[derive(Component, Debug, Default)]
pub struct TurretState {
    pub emp_timer: f32,   // > 0 = disabled
    pub fire_timer: f32,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,   // shots/second
}

// ---------------------------------------------------------------------------
// NPC
// ---------------------------------------------------------------------------

/// Role yang bisa dimiliki NPC — menentukan behavior hariannya.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcRole {
    Farmer,
    Builder,
    Guard,
    Healer,    // post-MVP
    Scavenger, // post-MVP
    Idle,      // baru rescue, belum di-assign
}

/// Komponen utama NPC survivor.
#[derive(Component, Debug, Clone)]
pub struct Npc {
    pub role: NpcRole,
    /// 0.0–1.0 hunger level (0 = lapar, 1 = kenyang).
    pub hunger: f32,
    /// 0.0–100.0 morale individu.
    pub morale: f32,
    /// HP NPC.
    pub hp: f32,
    /// Apakah NPC sedang assigned ke struktur tertentu.
    pub assigned_to: Option<Entity>,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            role: NpcRole::Idle,
            hunger: 1.0,
            morale: 70.0,
            hp: 40.0,
            assigned_to: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase Timer
// ---------------------------------------------------------------------------

/// Fase saat ini dalam siklus Day/Night.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    #[default]
    Day,
    Night,
}

/// Resource global yang menyimpan state fase dan timer.
#[derive(Resource, Debug, Clone)]
pub struct PhaseTimer {
    pub phase: Phase,
    /// Waktu tersisa dalam fase ini (detik).
    pub remaining: f32,
    /// Nomor hari saat ini (mulai dari 1).
    pub day: u32,
    /// Nomor wave malam ini.
    pub wave_num: u32,
}

impl Default for PhaseTimer {
    fn default() -> Self {
        Self {
            phase: Phase::Day,
            remaining: 180.0, // 3 menit
            day: 1,
            wave_num: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Strategy Tracker
// ---------------------------------------------------------------------------

/// Resource yang merekam pola bermain player untuk adaptive enemy system.
#[derive(Resource, Debug, Clone, Default)]
pub struct StrategyTracker {
    // Akumulasi damage ke wall vs total structure damage
    pub wall_damage_total: f32,
    pub structure_damage_total: f32,
    // Kill yang dilakukan turret vs total kills
    pub turret_kills: u32,
    pub total_kills: u32,
    // Waktu player diam (tidak bergerak) vs total malam
    pub stationary_time: f32,
    pub total_night_time: f32,
    // Kill yang dilakukan Guard NPC
    pub npc_guard_kills: u32,
    /// Night terakhir adaptation dievaluasi.
    pub last_evaluated_wave: u32,
}

impl StrategyTracker {
    pub fn wall_reliance(&self) -> f32 {
        if self.structure_damage_total == 0.0 {
            return 0.0;
        }
        self.wall_damage_total / self.structure_damage_total
    }

    pub fn turret_kill_ratio(&self) -> f32 {
        if self.total_kills == 0 {
            return 0.0;
        }
        self.turret_kills as f32 / self.total_kills as f32
    }

    pub fn stationary_ratio(&self) -> f32 {
        if self.total_night_time == 0.0 {
            return 0.0;
        }
        self.stationary_time / self.total_night_time
    }

    pub fn npc_kill_ratio(&self) -> f32 {
        if self.total_kills == 0 {
            return 0.0;
        }
        self.npc_guard_kills as f32 / self.total_kills as f32
    }
}

// ---------------------------------------------------------------------------
// Meta Progression
// ---------------------------------------------------------------------------

/// Persisted between runs. Serialized to JSON on disk.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaProgress {
    pub void_shards: u32,
    pub unlocked_blueprints: Vec<String>,
    /// Enemy types yang sudah pernah ditemui (untuk Codex — post-MVP).
    pub codex_seen: Vec<String>,
    // Unlock flags
    pub has_starting_turret: bool,
    pub has_starting_walls: bool,
    pub void_affinity_active: bool,
    pub colony_bond_active: bool,
    pub sentinel_unlocked: bool,
    pub wraith_unlocked: bool,    // post-MVP
    pub void_surge_unlocked: bool, // post-MVP
}

// ---------------------------------------------------------------------------
// Game State Resources
// ---------------------------------------------------------------------------

/// Colony-level stats: food stock, morale rata-rata, populasi.
#[derive(Resource, Debug, Clone)]
pub struct ColonyState {
    pub food: u32,
    pub morale: f32,       // 0.0–100.0
    pub population: u32,
    pub max_population: u32, // Houses × 4
    /// Berapa hari berturut-turut food = 0.
    pub starvation_days: u32,
}

impl Default for ColonyState {
    fn default() -> Self {
        Self {
            food: 10, // starter food
            morale: 70.0,
            population: 2,
            max_population: 0, // diupdate saat house dibangun
            starvation_days: 0,
        }
    }
}

/// Inventori resource player saat ini.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerResources {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
    pub food: u32, // alias ke ColonyState.food — diupdate bersama
}

/// Run-level statistics (untuk run-end screen dan Void Shard calculation).
#[derive(Resource, Debug, Clone, Default)]
pub struct RunStats {
    pub days_survived: u32,
    pub total_kills: u32,
    pub bosses_defeated: u32,
    pub npcs_alive: u32,
    pub structures_built: u32,
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Marker untuk Void Core entity — target utama musuh.
#[derive(Component)]
pub struct VoidCore;

/// Marker untuk resource node di peta.
#[derive(Component, Debug, Clone)]
pub struct ResourceNode {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
    pub food: u32,
}

/// Marker untuk projectile (turret, grenade).
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub piercing: bool,
    pub from_turret: bool,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Dispatch saat fase berganti (Day → Night atau Night → Day).
#[derive(Event, Debug, Clone)]
pub struct PhaseChanged {
    pub new_phase: Phase,
    pub day: u32,
    pub wave_num: u32,
}

/// Dispatch saat entity menerima damage.
#[derive(Event, Debug, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub from_turret: bool,
    pub from_npc: bool,
}

/// Dispatch saat enemy mati.
#[derive(Event, Debug, Clone)]
pub struct EnemyDied {
    pub entity: Entity,
    pub position: Vec2,
    pub exp_reward: u32,
    pub from_turret: bool,
    pub from_npc: bool,
}

/// Dispatch saat player naik level.
#[derive(Event, Debug, Clone)]
pub struct PlayerLeveledUp {
    pub new_level: u32,
}

/// Dispatch saat Void Core terkena damage.
#[derive(Event, Debug, Clone)]
pub struct VoidCoreDamaged {
    pub remaining_fraction: f32,
}

/// Dispatch saat NPC mati atau desersi.
#[derive(Event, Debug, Clone)]
pub enum NpcLost {
    Died { entity: Entity },
    Deserted { entity: Entity },
}
