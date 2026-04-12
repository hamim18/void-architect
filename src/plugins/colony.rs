// Void Architect — plugins/colony.rs
// Sprint 2: Colony Systems (S2-01 hingga S2-09)
//
// [S2-01] NPC entity — spawn, wander AI, role assignment, 2 NPC awal
// [S2-02] Farmer role — assign ke Farm, +2 Food/day
// [S2-03] Builder role — modifier konstruksi -30% (via resource BuilderBonus)
// [S2-04] Guard role — patrol zone, attack enemy terdekat, 40HP
// [S2-05] Hunger system — 1-2 Food/NPC/day tiap transisi Day
// [S2-06] Starvation — Day1 = -10 morale, Day2 = 1 NPC deserts, Day3 = 1 NPC dies
// [S2-07] Morale system — 0-100, event hooks, efficiency modifier <30
// [S2-08] House structure — 4 NPC cap per house, blokir rescue kalau penuh
// [S2-09] NPC rescue event — prompt [R/Skip] saat Day, 5s timeout
//
// ADR: Guard attack menggunakan satu query Enemy (sesuai ADR-15) — tidak spawn
// system terpisah per tipe NPC agar menghindari B0001 mutable query conflict.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::{
    ColonyState, DamageEvent, Enemy, FarmMarker, Health, HouseMarker,
    Npc, NpcLost, NpcRole, Phase, PhaseChanged, PlayerResources,
};
use crate::GameState;
use crate::plugins::player::PlayerMarker;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Ukuran sprite NPC
const NPC_SIZE: f32 = 10.0;

// Wander — NPC bergerak ke titik acak sekitar area
const NPC_WANDER_SPEED: f32 = 50.0;
const NPC_WANDER_RADIUS: f32 = 80.0;     // radius dari titik spawn
const NPC_WANDER_INTERVAL: f32 = 3.0;    // detik antar ganti tujuan wander

// Guard
const GUARD_SPEED: f32 = 75.0;
const GUARD_ATTACK_RANGE: f32 = 35.0;
const GUARD_DAMAGE: f32 = 12.0;
const GUARD_PATROL_RADIUS: f32 = 100.0;
const GUARD_ATTACK_COOLDOWN: f32 = 1.2;  // detik

// Hunger — food per hari per role (sesuai GDD 3.5)
const FOOD_COST_DEFAULT: u32 = 1;        // Farmer, Builder, Scavenger, Idle
const FOOD_COST_GUARD: u32 = 2;          // Guard, Healer = 2

// Morale threshold
const MORALE_LOW: f32 = 30.0;
const MORALE_HIGH: f32 = 80.0;
const MORALE_MAX: f32 = 100.0;

// Rescue
const RESCUE_PROMPT_TIMEOUT: f32 = 5.0;  // detik sebelum prompt hilang
const RESCUE_DETECT_RADIUS: f32 = 80.0;  // jarak player ke rescue trigger
const RESCUE_TRIGGER_INTERVAL: f32 = 60.0; // minimal detik antar rescue event

// Farmer bonus food (di luar produksi Farm structure yang ada di structures.rs)
const FARMER_BONUS_FOOD: u32 = 2;

// Warna NPC per role
const COLOR_NPC_IDLE: Color    = Color::srgb(0.50, 0.80, 0.80);
const COLOR_NPC_FARMER: Color  = Color::srgb(0.40, 0.75, 0.25);
const COLOR_NPC_BUILDER: Color = Color::srgb(0.90, 0.55, 0.10);
const COLOR_NPC_GUARD: Color   = Color::srgb(0.15, 0.55, 0.55);

// Posisi spawn 2 NPC awal (dekat Void Core di tengah map)
const NPC_SPAWN_POSITIONS: [(f32, f32); 2] = [
    (-50.0, -30.0),
    ( 50.0, -30.0),
];

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ColonyPlugin;

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<NpcLost>()
            .add_event::<NpcRescued>()
            .add_event::<MoraleChanged>()
            .insert_resource(RescueState::default())
            .insert_resource(BuilderBonus::default())
            // Spawn 2 NPC awal saat run dimulai
            .add_systems(OnEnter(GameState::InRun), (
                spawn_starting_npcs,
                reset_rescue_state,
            ))
            // Update systems tiap frame
            .add_systems(Update, (
                npc_wander_ai,          // S2-01: wander untuk Idle/Farmer/Builder
                npc_farmer_assign,      // S2-02: Farmer → assign ke Farm
                npc_guard_ai,           // S2-04: Guard patrol + attack
                update_builder_bonus,   // S2-03: hitung efficiency Builder
                update_house_cap,       // S2-08: hitung max_population dari House
                rescue_event_system,    // S2-09: spawn rescue prompt + handle input
                tick_rescue_timeout,    // S2-09: timeout prompt
            ).run_if(in_state(GameState::InRun)))
            // Phase-triggered systems (tiap Day/Night transition)
            .add_systems(Update, (
                hunger_system,          // S2-05: potong food tiap Day
                morale_daily_events,    // S2-07: morale event tiap Day/Night
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Events tambahan
// ---------------------------------------------------------------------------

/// Dipancarkan saat NPC berhasil diselamatkan (prompt [R] ditekan)
#[derive(Event, Debug, Clone)]
pub struct NpcRescued {
    pub npc_entity: Entity,
    pub role: NpcRole,
}

/// Dipancarkan saat morale berubah signifikan
#[derive(Event, Debug, Clone)]
pub struct MoraleChanged {
    pub old_value: f32,
    pub new_value: f32,
    pub reason: MoraleReason,
}

#[derive(Debug, Clone)]
pub enum MoraleReason {
    FoodSufficient,
    FoodShortage,
    NpcDied,
    WellBonus,          // placeholder — Well structure aktif di v0.2
    PlayerLevelUp,      // dipicu dari progression.rs via event
    VoidCoreDamaged,
    StructureDestroyed,
    BossDefeated,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Bonus efisiensi dari Builder — dipakai structures.rs untuk kurangi cost/time.
/// Diupdate tiap frame berdasarkan jumlah Builder aktif.
#[derive(Resource, Debug, Default, Clone)]
pub struct BuilderBonus {
    /// Berapa persen pengurangan waktu konstruksi (0.0 = tidak ada, 0.3 = -30%)
    pub construction_speed_bonus: f32,
}

/// Status rescue event yang sedang berjalan
#[derive(Resource, Debug, Default)]
pub struct RescueState {
    /// Ada rescue prompt aktif sekarang?
    pub prompt_active: bool,
    /// Timer countdown timeout prompt
    pub timeout_timer: f32,
    /// Entity rescue trigger (marker di dunia)
    pub trigger_entity: Option<Entity>,
    /// Cooldown sebelum rescue bisa muncul lagi
    pub next_rescue_cooldown: f32,
    /// Sudah ada 1 rescue hari ini?
    pub rescued_today: bool,
    /// Role NPC yang akan direscue (random dari pool)
    pub pending_role: NpcRole,
}

// ---------------------------------------------------------------------------
// Marker & NPC sub-components
// ---------------------------------------------------------------------------

/// Marker untuk rescue trigger — titik di map yang bisa di-rescue
#[derive(Component)]
pub struct RescueTrigger {
    pub role: NpcRole,
}

/// State wander NPC — target wander sekarang dan timer ganti target
#[derive(Component, Debug, Default)]
pub struct NpcWander {
    pub target: Vec2,
    pub timer: f32,
    /// Posisi home (titik tengah wander) — diset saat spawn
    pub home: Vec2,
}

/// State Guard — cooldown serangan dan last target
#[derive(Component, Debug, Default)]
pub struct GuardState {
    pub attack_timer: f32,
    pub patrol_target: Vec2,
    pub patrol_timer: f32,
}

/// Marker: NPC ini sedang assigned ke sebuah structure (Farm, dll)
#[derive(Component, Debug, Clone)]
pub struct AssignedToStructure {
    pub structure_entity: Entity,
}

// ---------------------------------------------------------------------------
// S2-01: Spawn 2 NPC Awal
// ---------------------------------------------------------------------------

/// Spawn 2 NPC awal saat run dimulai: 1 Farmer + 1 Builder
fn spawn_starting_npcs(
    mut commands: Commands,
    mut colony: ResMut<ColonyState>,
) {
    let initial_roles = [NpcRole::Farmer, NpcRole::Builder];

    for (i, role) in initial_roles.iter().enumerate() {
        let (x, y) = NPC_SPAWN_POSITIONS[i];
        let home = Vec2::new(x, y);

        spawn_npc(&mut commands, home, *role);
        colony.population += 1;
    }

    // Kosongkan — population sudah di-set saat spawn
    // colony.population dimulai dari 0 di reset, jadi kita tambah di sini
    bevy::log::info!(
        "[Colony] Spawn 2 NPC awal: Farmer + Builder. Population: {}",
        colony.population
    );
}

/// Helper: spawn satu NPC entity dengan role dan posisi tertentu
fn spawn_npc(commands: &mut Commands, pos: Vec2, role: NpcRole) -> Entity {
    let color = npc_color(role);
    let hp = match role {
        NpcRole::Guard => 40.0,
        _ => 20.0,
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(NPC_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
            ..default()
        },
        Npc {
            role,
            hunger: 1.0,
            morale: 70.0,
            hp,
            assigned_to: None,
        },
        NpcWander {
            target: pos,
            timer: 0.0,
            home: pos,
        },
        Health::new(hp),
        // Rapier collider untuk deteksi enemy attack ke NPC
        RigidBody::KinematicVelocityBased,
        Collider::ball(NPC_SIZE / 2.0),
        Velocity::zero(),
        ActiveEvents::COLLISION_EVENTS,
    )).id()
}

fn npc_color(role: NpcRole) -> Color {
    match role {
        NpcRole::Farmer    => COLOR_NPC_FARMER,
        NpcRole::Builder   => COLOR_NPC_BUILDER,
        NpcRole::Guard     => COLOR_NPC_GUARD,
        _                  => COLOR_NPC_IDLE,
    }
}

fn reset_rescue_state(mut rescue: ResMut<RescueState>) {
    *rescue = RescueState::default();
    rescue.next_rescue_cooldown = 30.0; // tunggu 30 detik sebelum rescue pertama
}

// ---------------------------------------------------------------------------
// S2-01: Wander AI — Idle / Farmer / Builder
// ---------------------------------------------------------------------------

/// NPC Idle, Farmer, Builder wander pelan ke titik acak sekitar home.
/// Guard punya sistem sendiri (npc_guard_ai).
fn npc_wander_ai(
    mut npc_q: Query<
        (&Npc, &mut NpcWander, &Transform, &mut Velocity),
        Without<Enemy>,
    >,
    time: Res<Time>,
    // Pakai SimpleRng via Local state — tidak perlu dependency rand tambahan
    mut rng: Local<SimpleRng>,
) {
    let dt = time.delta_seconds();

    for (npc, mut wander, tf, mut vel) in &mut npc_q {
        // Guard diurus oleh npc_guard_ai
        if npc.role == NpcRole::Guard { continue; }

        // Countdown timer ganti target
        wander.timer -= dt;

        if wander.timer <= 0.0 {
            // Pilih target baru: titik acak dalam radius dari home
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let dist  = rng.next_f32() * NPC_WANDER_RADIUS;
            wander.target = wander.home + Vec2::new(angle.cos() * dist, angle.sin() * dist);
            wander.timer  = NPC_WANDER_INTERVAL + rng.next_f32() * 1.5;
        }

        // Gerak menuju target wander
        let pos    = tf.translation.truncate();
        let to_tgt = wander.target - pos;
        let dist   = to_tgt.length();

        if dist > 6.0 {
            let dir = to_tgt / dist;
            vel.linvel = dir * NPC_WANDER_SPEED;
        } else {
            vel.linvel = Vec2::ZERO;
        }
    }
}

// ---------------------------------------------------------------------------
// S2-02: Farmer Role — Assign ke Farm + Bonus Food
// ---------------------------------------------------------------------------

/// Farmer yang belum assigned akan coba assign ke Farm structure terdekat.
/// Bonus food (+2/day) diberikan oleh hunger_system saat count Farmer.
fn npc_farmer_assign(
    mut npc_q: Query<(Entity, &mut Npc, &Transform, &mut NpcWander)>,
    farm_q: Query<(Entity, &Transform), With<FarmMarker>>,
) {
    for (npc_ent, mut npc, npc_tf, mut wander) in &mut npc_q {
        if npc.role != NpcRole::Farmer { continue; }
        if npc.assigned_to.is_some() { continue; }

        // Cari Farm terdekat yang belum ada NPC-nya (simplified — satu farmer per farm)
        let npc_pos = npc_tf.translation.truncate();

        let closest_farm = farm_q.iter()
            .min_by(|(_, a), (_, b)| {
                let da = npc_pos.distance(a.translation.truncate());
                let db = npc_pos.distance(b.translation.truncate());
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((farm_ent, farm_tf)) = closest_farm {
            npc.assigned_to = Some(farm_ent);
            // Update home wander ke sekitar farm
            wander.home = farm_tf.translation.truncate();
            bevy::log::info!("[Colony] Farmer {:?} assigned ke Farm {:?}", npc_ent, farm_ent);
        }
    }
}

// ---------------------------------------------------------------------------
// S2-03: Builder Bonus — Update resource BuilderBonus
// ---------------------------------------------------------------------------

/// Hitung jumlah Builder aktif, update BuilderBonus resource.
/// structures.rs bisa baca resource ini untuk modifier cost/waktu.
fn update_builder_bonus(
    npc_q: Query<&Npc>,
    mut bonus: ResMut<BuilderBonus>,
) {
    let builder_count = npc_q.iter()
        .filter(|n| n.role == NpcRole::Builder)
        .count();

    // 1 Builder = -30%. Multiple Builder tidak stack (cap di 30% sesuai GDD)
    bonus.construction_speed_bonus = if builder_count > 0 { 0.30 } else { 0.0 };
}

// ---------------------------------------------------------------------------
// S2-04: Guard Role — Patrol + Attack Enemy
// ---------------------------------------------------------------------------

/// Guard: kalau ada enemy dalam range → attack. Kalau tidak → patrol sekitar home.
fn npc_guard_ai(
    mut guard_q: Query<(Entity, &Npc, &mut GuardState, &mut Velocity, &Transform, &mut Sprite)>,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
    mut strategy: ResMut<crate::components::StrategyTracker>,
    time: Res<Time>,
    mut rng: Local<SimpleRng>,
) {
    let dt = time.delta_seconds();

    // Kumpulkan posisi semua enemy sekali — hindari borrow conflict
    let enemies: Vec<(Entity, Vec2)> = enemy_q.iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .collect();

    for (_guard_ent, npc, mut state, mut vel, tf, mut sprite) in &mut guard_q {
        if npc.role != NpcRole::Guard { continue; }

        state.attack_timer  = (state.attack_timer  - dt).max(0.0);
        state.patrol_timer  = (state.patrol_timer  - dt).max(0.0);

        let guard_pos = tf.translation.truncate();

        // Cari enemy dalam GUARD_ATTACK_RANGE + GUARD_PATROL_RADIUS
        let closest_enemy = enemies.iter()
            .filter(|(_, epos)| guard_pos.distance(*epos) <= GUARD_PATROL_RADIUS)
            .min_by(|(_, a), (_, b)| {
                guard_pos.distance(*a)
                    .partial_cmp(&guard_pos.distance(*b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((enemy_ent, enemy_pos)) = closest_enemy {
            // Ada enemy dalam zona — kejar dan attack
            let to_enemy = *enemy_pos - guard_pos;
            let dist     = to_enemy.length();

            if dist > GUARD_ATTACK_RANGE {
                // Kejar enemy
                vel.linvel = (to_enemy / dist) * GUARD_SPEED;
                sprite.color = COLOR_NPC_GUARD.lighter(0.1); // sedikit terang saat aktif
            } else {
                // Dalam range — stop dan attack
                vel.linvel = Vec2::ZERO;

                if state.attack_timer <= 0.0 {
                    damage_events.send(DamageEvent {
                        target: *enemy_ent,
                        amount: GUARD_DAMAGE,
                        from_turret: false,
                        from_npc: true,
                    });
                    state.attack_timer = GUARD_ATTACK_COOLDOWN;
                    strategy.npc_guard_kills += 0; // kill dihitung di combat.rs saat enemy mati
                }
            }
        } else {
            // Tidak ada enemy — patrol pelan ke titik acak sekitar home wander
            sprite.color = COLOR_NPC_GUARD;

            if state.patrol_timer <= 0.0 {
                let angle = rng.next_f32() * std::f32::consts::TAU;
                let dist  = rng.next_f32() * GUARD_PATROL_RADIUS * 0.5;
                state.patrol_target = guard_pos + Vec2::new(angle.cos() * dist, angle.sin() * dist);
                state.patrol_timer  = 3.0 + rng.next_f32() * 2.0;
            }

            let to_patrol = state.patrol_target - guard_pos;
            let dist      = to_patrol.length();
            if dist > 8.0 {
                vel.linvel = (to_patrol / dist) * NPC_WANDER_SPEED;
            } else {
                vel.linvel = Vec2::ZERO;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// S2-08: House Cap — Hitung max_population dari House structures
// ---------------------------------------------------------------------------

const HOUSE_CAPACITY: u32 = 4;

fn update_house_cap(
    house_q: Query<(), With<HouseMarker>>,
    mut colony: ResMut<ColonyState>,
) {
    // Setiap House = 4 kapasitas
    let house_count = house_q.iter().count() as u32;
    colony.max_population = house_count * HOUSE_CAPACITY;

    // Minimum cap: 2 (dari 2 NPC awal bisa bertahan tanpa house)
    // Jika tidak ada house sama sekali, set cap ke 2 agar game bisa dimulai
    if colony.max_population == 0 {
        colony.max_population = 2;
    }
}

// ---------------------------------------------------------------------------
// S2-05: Hunger System — Potong Food tiap Day transition
// ---------------------------------------------------------------------------

/// Dijalankan saat PhaseChanged(Day) — potong food sesuai jumlah NPC.
fn hunger_system(
    mut phase_events: EventReader<PhaseChanged>,
    npc_q: Query<&Npc>,
    mut colony: ResMut<ColonyState>,
    mut resources: ResMut<PlayerResources>,
    mut npc_lost_events: EventWriter<NpcLost>,
    mut morale_events: EventWriter<MoraleChanged>,
    mut commands: Commands,
    // Baca entitas NPC untuk despawn saat starvation hari ke-3
    npc_entity_q: Query<(Entity, &Npc)>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Day { continue; }

        // --- Hitung total food yang dibutuhkan ---
        let mut food_needed: u32 = 0;
        let mut farmer_count: u32 = 0;

        for npc in &npc_q {
            food_needed += match npc.role {
                NpcRole::Guard | NpcRole::Healer => FOOD_COST_GUARD,
                _ => FOOD_COST_DEFAULT,
            };
            if npc.role == NpcRole::Farmer {
                farmer_count += 1;
            }
        }

        // Farmer bonus food (di luar Farm structure production)
        let farmer_bonus = farmer_count * FARMER_BONUS_FOOD;
        // Bonus ditambah ke colony food (Farm production sudah ditambah di structures.rs)
        if farmer_bonus > 0 {
            resources.food         = resources.food.saturating_add(farmer_bonus);
            colony.food             = colony.food.saturating_add(farmer_bonus);
        }

        bevy::log::info!(
            "[Colony] Day {} — food needed: {}, available: {}, farmer bonus: {}",
            ev.day, food_needed, colony.food, farmer_bonus
        );

        if colony.food >= food_needed {
            // --- Cukup makan ---
            colony.food         = colony.food.saturating_sub(food_needed);
            resources.food       = resources.food.saturating_sub(food_needed.min(resources.food));
            colony.starvation_days = 0;

            // Morale +3 per hari jika cukup makan
            apply_morale_delta(&mut colony, 3.0, MoraleReason::FoodSufficient, &mut morale_events);

        } else {
            // --- Kekurangan makanan ---
            colony.food         = 0;
            resources.food       = 0;
            colony.starvation_days += 1;

            let starvation_day = colony.starvation_days;
            bevy::log::warn!(
                "[Colony] STARVATION day {}! Population: {}",
                starvation_day, colony.population
            );

            // S2-06: Starvation consequences
            starvation_consequence(
                starvation_day,
                &mut colony,
                &mut commands,
                &npc_entity_q,
                &mut npc_lost_events,
                &mut morale_events,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// S2-06: Starvation Consequences
// ---------------------------------------------------------------------------

fn starvation_consequence(
    starvation_day: u32,
    colony: &mut ColonyState,
    commands: &mut Commands,
    npc_entity_q: &Query<(Entity, &Npc)>,
    npc_lost_events: &mut EventWriter<NpcLost>,
    morale_events: &mut EventWriter<MoraleChanged>,
) {
    // Day 1 kelaparan: morale -10
    apply_morale_delta(colony, -10.0, MoraleReason::FoodShortage, morale_events);

    if starvation_day >= 2 {
        // Day 2+: 1 NPC desersi
        // Pilih NPC non-Guard pertama (Guard terakhir meninggalkan koloni)
        let deserter = npc_entity_q.iter()
            .find(|(_, npc)| npc.role != NpcRole::Guard)
            .map(|(e, _)| e)
            .or_else(|| npc_entity_q.iter().next().map(|(e, _)| e));

        if let Some(ent) = deserter {
            commands.entity(ent).despawn_recursive();
            colony.population = colony.population.saturating_sub(1);
            npc_lost_events.send(NpcLost::Deserted { entity: ent });
            bevy::log::warn!("[Colony] NPC {:?} DESERSI karena kelaparan!", ent);
        }
    }

    if starvation_day >= 3 {
        // Day 3+: 1 NPC tambahan mati
        let victim = npc_entity_q.iter()
            .next()
            .map(|(e, _)| e);

        if let Some(ent) = victim {
            commands.entity(ent).despawn_recursive();
            colony.population = colony.population.saturating_sub(1);
            npc_lost_events.send(NpcLost::Died { entity: ent });
            apply_morale_delta(colony, -10.0, MoraleReason::NpcDied, morale_events);
            bevy::log::warn!("[Colony] NPC {:?} MATI karena kelaparan!", ent);
        }
    }
}

// ---------------------------------------------------------------------------
// S2-07: Morale Daily Events
// ---------------------------------------------------------------------------

/// Morale event yang di-trigger saat transisi Day/Night.
/// Morale point-based di luar hunger sudah di-handle hunger_system.
fn morale_daily_events(
    mut phase_events: EventReader<PhaseChanged>,
    colony: Res<ColonyState>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Day { continue; }
        bevy::log::info!(
            "[Colony] Day {} — Morale: {:.1}/100 | Population: {}/{}",
            ev.day, colony.morale, colony.population, colony.max_population
        );
    }
}

// ---------------------------------------------------------------------------
// S2-09: NPC Rescue Event
// ---------------------------------------------------------------------------

/// Spawn rescue trigger di titik acak dekat tepi map saat Day phase dimulai.
fn rescue_event_system(
    mut commands: Commands,
    mut rescue: ResMut<RescueState>,
    mut phase_events: EventReader<PhaseChanged>,
    player_q: Query<&Transform, With<PlayerMarker>>,
    rescue_trigger_q: Query<(Entity, &Transform), With<RescueTrigger>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    // Pakai ResMut saja — Res + ResMut ke resource yang sama dalam satu system = compile error
    mut colony_mut: ResMut<ColonyState>,
    mut npc_rescued_events: EventWriter<NpcRescued>,
    time: Res<Time>,
    mut rng: Local<SimpleRng>,
) {
    let dt = time.delta_seconds();

    // Hitung ulang cooldown rescue antar hari
    rescue.next_rescue_cooldown = (rescue.next_rescue_cooldown - dt).max(0.0);

    // Cek apakah phase baru adalah Day — spawn rescue trigger baru
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Day { continue; }

        // Reset rescued_today tiap pagi
        rescue.rescued_today = false;

        // Jangan spawn kalau population sudah max atau cooldown belum habis
        if colony_mut.population < colony_mut.max_population
            && rescue.next_rescue_cooldown <= 0.0
        {
            spawn_rescue_trigger(&mut commands, &mut rescue, &mut rng);
        }
    }

    // Jika tidak ada prompt aktif, cek apakah player dekat rescue trigger
    if !rescue.prompt_active {
        let Ok(player_tf) = player_q.get_single() else { return };
        let player_pos = player_tf.translation.truncate();

        for (trigger_ent, trigger_tf) in &rescue_trigger_q {
            let dist = player_pos.distance(trigger_tf.translation.truncate());
            if dist <= RESCUE_DETECT_RADIUS && !rescue.rescued_today {
                // Aktifkan prompt
                rescue.prompt_active   = true;
                rescue.timeout_timer   = RESCUE_PROMPT_TIMEOUT;
                rescue.trigger_entity  = Some(trigger_ent);
                bevy::log::info!(
                    "[Colony] Rescue prompt aktif! Role: {:?} | Tekan [R] untuk rescue, [N] untuk skip",
                    rescue.pending_role
                );
            }
        }
        return;
    }

    // Prompt sedang aktif — handle input
    if keyboard.just_pressed(KeyCode::KeyR) {
        // [R] — Rescue NPC
        complete_rescue(&mut commands, &mut rescue, &mut colony_mut, &mut npc_rescued_events, &mut rng);
    } else if keyboard.just_pressed(KeyCode::KeyN) {
        // [N/Skip] — Tolak rescue
        dismiss_rescue(&mut commands, &mut rescue);
        bevy::log::info!("[Colony] Rescue di-skip.");
    }
}

/// Spawn entitas marker rescue trigger di posisi acak (pinggir tengah map)
fn spawn_rescue_trigger(
    commands: &mut Commands,
    rescue: &mut RescueState,
    rng: &mut SimpleRng,
) {
    // Posisi acak di sekitar area bermain (simplified)
    let angle = rng.next_f32() * std::f32::consts::TAU;
    let dist  = 150.0 + rng.next_f32() * 200.0;
    let pos   = Vec2::new(angle.cos() * dist, angle.sin() * dist);

    // Pilih role acak (Farmer/Builder/Guard dengan probabilitas berbeda)
    let role = match rng.next_u32() % 3 {
        0 => NpcRole::Farmer,
        1 => NpcRole::Builder,
        _ => NpcRole::Guard,
    };

    rescue.pending_role = role;

    let ent = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.9, 0.2), // kuning — marker rescue
                custom_size: Some(Vec2::splat(12.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.8),
            ..default()
        },
        RescueTrigger { role },
    )).id();

    rescue.trigger_entity = Some(ent);
    bevy::log::info!(
        "[Colony] Rescue trigger spawn: {:?} role {:?} di {:?}",
        ent, role, pos
    );
}

/// Konfirmasi rescue — spawn NPC baru dan update state
fn complete_rescue(
    commands: &mut Commands,
    rescue: &mut RescueState,
    colony: &mut ColonyState,
    npc_rescued_events: &mut EventWriter<NpcRescued>,
    rng: &mut SimpleRng,
) {
    // Hapus trigger entity
    if let Some(trigger_ent) = rescue.trigger_entity.take() {
        commands.entity(trigger_ent).despawn_recursive();
    }

    // Spawn NPC baru di posisi dekat Void Core (tengah map)
    let spawn_offset = Vec2::new(
        (rng.next_f32() - 0.5) * 60.0,
        (rng.next_f32() - 0.5) * 60.0,
    );
    let npc_ent = spawn_npc(commands, spawn_offset, rescue.pending_role);

    // Tambah GuardState component kalau Guard
    if rescue.pending_role == NpcRole::Guard {
        commands.entity(npc_ent).insert(GuardState::default());
    }

    colony.population += 1;
    rescue.prompt_active  = false;
    rescue.rescued_today  = true;
    rescue.next_rescue_cooldown = RESCUE_TRIGGER_INTERVAL;

    npc_rescued_events.send(NpcRescued {
        npc_entity: npc_ent,
        role: rescue.pending_role,
    });

    bevy::log::info!(
        "[Colony] NPC {:?} diselamatkan! Role: {:?}. Population: {}/{}",
        npc_ent, rescue.pending_role, colony.population, colony.max_population
    );
}

/// Dismiss prompt tanpa rescue
fn dismiss_rescue(commands: &mut Commands, rescue: &mut RescueState) {
    if let Some(trigger_ent) = rescue.trigger_entity.take() {
        commands.entity(trigger_ent).despawn_recursive();
    }
    rescue.prompt_active = false;
    rescue.next_rescue_cooldown = RESCUE_TRIGGER_INTERVAL * 0.5; // cooldown lebih pendek saat skip
}

/// Countdown timeout prompt rescue — auto-dismiss kalau tidak direspons
fn tick_rescue_timeout(
    mut commands: Commands,
    mut rescue: ResMut<RescueState>,
    time: Res<Time>,
) {
    if !rescue.prompt_active { return; }

    rescue.timeout_timer -= time.delta_seconds();

    if rescue.timeout_timer <= 0.0 {
        dismiss_rescue(&mut commands, &mut rescue);
        bevy::log::info!("[Colony] Rescue prompt timeout — auto-skip.");
    }
}

// ---------------------------------------------------------------------------
// Morale Helper — dipakai beberapa sistem
// ---------------------------------------------------------------------------

/// Terapkan delta morale ke ColonyState dan kirim event.
pub fn apply_morale_delta(
    colony: &mut ColonyState,
    delta: f32,
    reason: MoraleReason,
    morale_events: &mut EventWriter<MoraleChanged>,
) {
    let old = colony.morale;
    colony.morale = (colony.morale + delta).clamp(0.0, MORALE_MAX);
    let new = colony.morale;

    if (old - new).abs() > 0.01 {
        morale_events.send(MoraleChanged { old_value: old, new_value: new, reason });
    }
}

/// Hitung efisiensi NPC berdasarkan morale koloni.
/// Dipakai sistem lain yang butuh modifier efisiensi.
pub fn npc_efficiency(colony: &ColonyState) -> f32 {
    if colony.morale < MORALE_LOW {
        0.70 // di bawah 30 morale: 70% efisiensi
    } else if colony.morale > MORALE_HIGH {
        1.20 // di atas 80 morale: +20% bonus efisiensi
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------
// SimpleRng — LCG internal (sama dengan enemies.rs, tidak perlu crate rand)
// ---------------------------------------------------------------------------

/// Linear Congruential Generator sederhana — tidak butuh dependency eksternal.
/// Seed dari waktu sistem saat inisialisasi.
#[derive(Default)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn next_u32(&mut self) -> u32 {
        // Knuth multiplicative hash
        if self.state == 0 {
            self.state = 6364136223846793005;
        }
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}
