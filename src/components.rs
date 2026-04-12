// Void Architect — components.rs
// Core ECS components. Didefinisikan di satu tempat untuk menghindari
// refactor besar-besaran antar plugin. Semua plugin import dari sini.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Spatial
// ---------------------------------------------------------------------------

/// 2D world position snapshot — shortcut reference di luar Transform.
/// Physics sebenarnya diurus rapier via Transform; ini hanya cache baca.
#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

// NOTE: Tidak ada Velocity di sini — rapier punya sendiri (bevy_rapier2d::Velocity).
// Kalau plugin lain butuh baca kecepatan, query bevy_rapier2d::prelude::Velocity langsung.

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

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

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub level: u32,
    pub exp: u32,
    pub exp_next: u32,
    pub morale_aura: f32,
    pub facing: f32,
    pub is_dashing: bool,
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

#[derive(Debug, Clone, Default)]
pub struct AbilityCooldowns {
    pub melee: f32,
    pub dash: f32,
    pub grenade: f32,
    pub void_explosion: f32,
    pub repair_pulse: f32,
}

// ---------------------------------------------------------------------------
// Enemy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    VoidDrone,
    RiftStalker,
    Breacher,
    VoidCrawler,
    HollowTitan,
    RiftHive,
    SwarmLord,
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub variant: EnemyType,
    pub adapt_flags: u8,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub exp_reward: u32,
}

pub mod adapt_flags {
    pub const TARGET_NPC: u8 = 0b0000_0001;
    pub const BYPASS_WALL: u8 = 0b0000_0010;
    pub const ENRAGED: u8 = 0b0000_0100;
}

// ---------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------

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

#[derive(Component, Debug, Clone)]
pub struct Structure {
    pub tier: u8,
    pub structure_type: StructureType,
    pub durability: f32,
    pub grid_pos: IVec2,
}

#[derive(Component)] pub struct WallMarker;
#[derive(Component)] pub struct TurretMarker;
#[derive(Component)] pub struct FarmMarker;
#[derive(Component)] pub struct HouseMarker;

#[derive(Component, Debug, Default)]
pub struct TurretState {
    pub emp_timer: f32,
    pub fire_timer: f32,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
}

// ---------------------------------------------------------------------------
// NPC
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcRole {
    Farmer,
    Builder,
    Guard,
    Healer,    // post-MVP
    Scavenger, // post-MVP
    Idle,
}

#[derive(Component, Debug, Clone)]
pub struct Npc {
    pub role: NpcRole,
    pub hunger: f32,
    pub morale: f32,
    pub hp: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    #[default]
    Day,
    Night,
}

#[derive(Resource, Debug, Clone)]
pub struct PhaseTimer {
    pub phase: Phase,
    pub remaining: f32,
    pub day: u32,
    pub wave_num: u32,
}

impl Default for PhaseTimer {
    fn default() -> Self {
        Self {
            phase: Phase::Day,
            remaining: 180.0,
            day: 1,
            wave_num: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Strategy Tracker
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Default)]
pub struct StrategyTracker {
    pub wall_damage_total: f32,
    pub structure_damage_total: f32,
    pub turret_kills: u32,
    pub total_kills: u32,
    pub stationary_time: f32,
    pub total_night_time: f32,
    pub npc_guard_kills: u32,
    pub last_evaluated_wave: u32,
}

impl StrategyTracker {
    pub fn wall_reliance(&self) -> f32 {
        if self.structure_damage_total == 0.0 { return 0.0; }
        self.wall_damage_total / self.structure_damage_total
    }

    pub fn turret_kill_ratio(&self) -> f32 {
        if self.total_kills == 0 { return 0.0; }
        self.turret_kills as f32 / self.total_kills as f32
    }

    pub fn stationary_ratio(&self) -> f32 {
        if self.total_night_time == 0.0 { return 0.0; }
        self.stationary_time / self.total_night_time
    }

    pub fn npc_kill_ratio(&self) -> f32 {
        if self.total_kills == 0 { return 0.0; }
        self.npc_guard_kills as f32 / self.total_kills as f32
    }
}

// ---------------------------------------------------------------------------
// Meta Progression
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaProgress {
    pub void_shards: u32,
    pub unlocked_blueprints: Vec<String>,
    pub codex_seen: Vec<String>,
    pub has_starting_turret: bool,
    pub has_starting_walls: bool,
    pub void_affinity_active: bool,
    pub colony_bond_active: bool,
    pub sentinel_unlocked: bool,
    pub wraith_unlocked: bool,
    pub void_surge_unlocked: bool,
}

// ---------------------------------------------------------------------------
// Game State Resources
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone)]
pub struct ColonyState {
    pub food: u32,
    pub morale: f32,
    pub population: u32,
    pub max_population: u32,
    pub starvation_days: u32,
}

impl Default for ColonyState {
    fn default() -> Self {
        Self {
            food: 10,
            morale: 70.0,
            population: 2,
            max_population: 0,
            starvation_days: 0,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerResources {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
    pub food: u32,
}

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

#[derive(Component)] pub struct VoidCore;

#[derive(Component, Debug, Clone)]
pub struct ResourceNode {
    pub stone: u32,
    pub scrap: u32,
    pub void_crystal: u32,
    pub food: u32,
}

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

#[derive(Event, Debug, Clone)]
pub struct PhaseChanged {
    pub new_phase: Phase,
    pub day: u32,
    pub wave_num: u32,
}

#[derive(Event, Debug, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub from_turret: bool,
    pub from_npc: bool,
}

#[derive(Event, Debug, Clone)]
pub struct EnemyDied {
    pub entity: Entity,
    pub position: Vec2,
    pub exp_reward: u32,
    pub from_turret: bool,
    pub from_npc: bool,
}

#[derive(Event, Debug, Clone)]
pub struct PlayerLeveledUp {
    pub new_level: u32,
}

#[derive(Event, Debug, Clone)]
pub struct VoidCoreDamaged {
    pub remaining_fraction: f32,
}

#[derive(Event, Debug, Clone)]
pub enum NpcLost {
    Died { entity: Entity },
    Deserted { entity: Entity },
}
