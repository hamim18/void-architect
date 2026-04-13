// Void Architect — plugins/structures.rs
// Wall, Turret, Farm placement, durability, turret AI, build mode.
// [S1-05, S1-06, S1-07, S1-08]
//
// Build Mode: tekan [B] untuk toggle. Ghost preview di cursor.
// LMB = place structure. ESC = cancel. Resource dipotong saat place.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::components::*;
use crate::GameState;
use crate::plugins::progression::{can_afford, spend_resources, ResourceCost};
use crate::plugins::world::TILE_SIZE;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Wall — S1-05
const WALL_HP: f32 = 100.0;
const WALL_COST: ResourceCost = ResourceCost { stone: 10, scrap: 0, void_crystal: 0 };

// Turret — S1-06
const TURRET_HP: f32 = 80.0;
const TURRET_RANGE: f32 = 150.0;
const TURRET_FIRE_RATE: f32 = 1.5; // detik antar tembakan
const TURRET_DAMAGE: f32 = 15.0;
const TURRET_PROJECTILE_SPEED: f32 = 280.0;
const TURRET_COST: ResourceCost = ResourceCost { stone: 30, scrap: 20, void_crystal: 0 };

// Farm — S1-07
const FARM_HP: f32 = 60.0;
const FARM_FOOD_PER_DAY: u32 = 3;
const FARM_COST: ResourceCost = ResourceCost { stone: 20, scrap: 0, void_crystal: 0 };

// House (untuk S2)
const HOUSE_HP: f32 = 80.0;
const HOUSE_COST: ResourceCost = ResourceCost { stone: 30, scrap: 0, void_crystal: 0 };

// Ghost preview opacity
const GHOST_ALPHA: f32 = 0.45;

// Warna struktur
const COLOR_WALL: Color         = Color::srgb(0.40, 0.50, 0.65);
const COLOR_WALL_BORDER: Color  = Color::srgb(0.60, 0.70, 0.85);
const COLOR_TURRET_BASE: Color  = Color::srgb(0.20, 0.70, 0.70);
const COLOR_TURRET_BARREL: Color = Color::srgb(0.10, 0.50, 0.50);
const COLOR_FARM: Color         = Color::srgb(0.25, 0.65, 0.25);
const COLOR_HOUSE: Color        = Color::srgb(0.60, 0.45, 0.25);
const COLOR_INVALID: Color      = Color::srgba(1.0, 0.2, 0.2, GHOST_ALPHA);
const COLOR_VALID: Color        = Color::srgba(0.5, 1.0, 0.5, GHOST_ALPHA);
const COLOR_PROJECTILE: Color   = Color::srgb(1.0, 0.8, 0.0);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BuildMode::default())
            .add_systems(Update, (
                // Build Mode systems
                toggle_build_mode,
                update_ghost_preview,
                place_structure,
                // Structure systems
                turret_ai,                 // S1-06: auto-target & fire
                turret_projectile_move,    // S1-06: projectile gerak
                farm_produce_food,         // S1-07: food production per day
                structure_damage_visual,   // visual feedback kerusakan
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Build Mode Resource
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSelection {
    Wall,
    Turret,
    Farm,
    House,
}

impl BuildSelection {
    pub fn cost(&self) -> ResourceCost {
        match self {
            BuildSelection::Wall   => WALL_COST,
            BuildSelection::Turret => TURRET_COST,
            BuildSelection::Farm   => FARM_COST,
            BuildSelection::House  => HOUSE_COST,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuildSelection::Wall   => "Wall",
            BuildSelection::Turret => "Turret",
            BuildSelection::Farm   => "Farm",
            BuildSelection::House  => "House",
        }
    }
}

#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub selected: Option<BuildSelection>,
    pub ghost_entity: Option<Entity>,
    /// FIX-02: Cooldown antar placement saat RMB di-hold (detik)
    pub place_cooldown: f32,
}

// ---------------------------------------------------------------------------
// Marker Components
// ---------------------------------------------------------------------------

/// Marker: ini adalah ghost preview (bukan real structure)
#[derive(Component)]
struct GhostPreview;

/// Turret barrel entity — child dari turret base, berputar ke target
#[derive(Component)]
struct TurretBarrel {
    pub turret_entity: Entity,
}

/// Turret projectile
#[derive(Component)]
struct TurretProjectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
}

// ---------------------------------------------------------------------------
// S1-08: Build Mode Toggle
// ---------------------------------------------------------------------------

fn toggle_build_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut build_mode: ResMut<BuildMode>,
    mut commands: Commands,
) {
    // [B] → toggle build mode (default: Wall)
    if keyboard.just_pressed(KeyCode::KeyB) {
        if build_mode.active {
            cancel_build_mode(&mut build_mode, &mut commands);
        } else {
            build_mode.active = true;
            build_mode.selected = Some(BuildSelection::Wall);
            bevy::log::info!("[Build] Mode ON | RMB=Place / Hold=Wall panjang | W=Wall T=Turret G=Farm H=House");
        }
    }

    // ESC → cancel build mode
    if keyboard.just_pressed(KeyCode::Escape) && build_mode.active {
        cancel_build_mode(&mut build_mode, &mut commands);
    }

    // Pilih jenis struktur saat build mode aktif
    if build_mode.active {
        if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::KeyW) {
            build_mode.selected = Some(BuildSelection::Wall);
        }
        if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::KeyT) {
            build_mode.selected = Some(BuildSelection::Turret);
        }
        if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::KeyG) {
            build_mode.selected = Some(BuildSelection::Farm);
        }
        if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::KeyH) {
            build_mode.selected = Some(BuildSelection::House);
        }
    }
}

fn cancel_build_mode(build_mode: &mut BuildMode, commands: &mut Commands) {
    build_mode.active = false;
    if let Some(ghost) = build_mode.ghost_entity.take() {
        commands.entity(ghost).despawn_recursive();
    }
    bevy::log::info!("[Build] Mode OFF");
}

// ---------------------------------------------------------------------------
// Ghost Preview Update
// ---------------------------------------------------------------------------

fn update_ghost_preview(
    mut commands: Commands,
    mut build_mode: ResMut<BuildMode>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    resources: Res<PlayerResources>,
    mut ghost_q: Query<(&mut Transform, &mut Sprite), With<GhostPreview>>,
) {
    if !build_mode.active { return; }
    let Some(selection) = build_mode.selected else { return };

    // Resolve cursor world pos
    let cursor_world: Option<Vec2> = (|| {
        let window = window_q.get_single().ok()?;
        let (camera, cam_gtf) = camera_q.get_single().ok()?;
        let cursor = window.cursor_position()?;
        camera.viewport_to_world_2d(cam_gtf, cursor)
    })();

    let Some(cursor_pos) = cursor_world else { return };

    // Snap ke grid
    let grid_pos = snap_to_grid(cursor_pos);
    let affordable = can_afford(&resources, &selection.cost());
    // FIX-03: Clamp grid pos ke dalam batas map
    let map_hw = crate::plugins::world::MAP_HALF_W - crate::plugins::world::TILE_SIZE;
    let map_hh = crate::plugins::world::MAP_HALF_H - crate::plugins::world::TILE_SIZE;
    let grid_pos = Vec2::new(
        grid_pos.x.clamp(-map_hw, map_hw),
        grid_pos.y.clamp(-map_hh, map_hh),
    );
    let ghost_color = if affordable { COLOR_VALID } else { COLOR_INVALID };
    let ghost_size = structure_size(selection);

    // Update atau spawn ghost
    if let Some(ghost_entity) = build_mode.ghost_entity {
        if let Ok((mut tf, mut sprite)) = ghost_q.get_mut(ghost_entity) {
            tf.translation.x = grid_pos.x;
            tf.translation.y = grid_pos.y;
            sprite.color = ghost_color;
            sprite.custom_size = Some(ghost_size);
        }
    } else {
        // Spawn ghost baru
        let ghost = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: ghost_color,
                    custom_size: Some(ghost_size),
                    ..default()
                },
                transform: Transform::from_xyz(grid_pos.x, grid_pos.y, 1.8),
                ..default()
            },
            GhostPreview,
        )).id();
        build_mode.ghost_entity = Some(ghost);
    }
}

// ---------------------------------------------------------------------------
// FIX-02: Place Structure — RMB klik atau hold (wall panjang)
// ---------------------------------------------------------------------------

/// Throttle saat hold RMB: jarak waktu minimum antar placement
const HOLD_PLACE_INTERVAL: f32 = 0.14; // ~7 wall per detik

fn place_structure(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut build_mode: ResMut<BuildMode>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut resources: ResMut<PlayerResources>,
    existing_q: Query<&Structure>,
    phase_timer: Res<PhaseTimer>,
    mut colony: ResMut<ColonyState>,
    time: Res<Time>,
) {
    if !build_mode.active { return; }
    if phase_timer.phase == Phase::Night { return; }

    let rmb_just = mouse.just_pressed(MouseButton::Right);
    let rmb_held = mouse.pressed(MouseButton::Right);
    if !rmb_just && !rmb_held { return; }

    // Tick cooldown
    build_mode.place_cooldown -= time.delta_seconds();

    // Kalau hold (bukan just press): skip kalau cooldown belum habis
    if rmb_held && !rmb_just && build_mode.place_cooldown > 0.0 { return; }

    let Some(selection) = build_mode.selected else { return };

    let cursor_world: Option<Vec2> = (|| {
        let window = window_q.get_single().ok()?;
        let (camera, cam_gtf) = camera_q.get_single().ok()?;
        let cursor = window.cursor_position()?;
        camera.viewport_to_world_2d(cam_gtf, cursor)
    })();

    let Some(cursor_pos) = cursor_world else { return };

    // FIX-03: Clamp ke dalam batas map
    let map_hw = crate::plugins::world::MAP_HALF_W - crate::plugins::world::TILE_SIZE;
    let map_hh = crate::plugins::world::MAP_HALF_H - crate::plugins::world::TILE_SIZE;
    let clamped = Vec2::new(cursor_pos.x.clamp(-map_hw, map_hw), cursor_pos.y.clamp(-map_hh, map_hh));

    let grid_pos = snap_to_grid(clamped);
    let grid_cell = world_to_grid(grid_pos);

    let occupied = existing_q.iter().any(|s| s.grid_pos == grid_cell);
    if occupied { return; } // skip diam-diam saat hold

    let cost = selection.cost();
    if !spend_resources(&mut resources, &cost) {
        if rmb_just { // hanya warn saat klik pertama, tidak saat hold
            bevy::log::warn!("[Build] Resource tidak cukup untuk {}", selection.name());
        }
        return;
    }

    match selection {
        BuildSelection::Wall   => spawn_wall(&mut commands, grid_pos, grid_cell),
        BuildSelection::Turret => spawn_turret(&mut commands, grid_pos, grid_cell),
        BuildSelection::Farm   => spawn_farm(&mut commands, grid_pos, grid_cell),
        BuildSelection::House  => spawn_house(&mut commands, grid_pos, grid_cell, &mut colony),
    }

    // Reset cooldown untuk hold berikutnya
    build_mode.place_cooldown = HOLD_PLACE_INTERVAL;

    bevy::log::info!(
        "[Build] {} di {:?} | Stone:{} Scrap:{}",
        selection.name(), grid_cell, resources.stone, resources.scrap
    );
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// S1-05: Spawn Wall
// ---------------------------------------------------------------------------

fn spawn_wall(commands: &mut Commands, world_pos: Vec2, grid_pos: IVec2) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_WALL,
                custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            ..default()
        },
        // Collider solid untuk block pathfinding musuh
        RigidBody::Fixed,
        Collider::cuboid((TILE_SIZE - 2.0) / 2.0, (TILE_SIZE - 2.0) / 2.0),
        Structure {
            tier: 1,
            structure_type: StructureType::Wall,
            durability: WALL_HP,
            grid_pos,
        },
        Health::new(WALL_HP),
        WallMarker,
    ));
}

// ---------------------------------------------------------------------------
// S1-06: Spawn Turret
// ---------------------------------------------------------------------------

fn spawn_turret(commands: &mut Commands, world_pos: Vec2, grid_pos: IVec2) {
    let turret_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_TURRET_BASE,
                custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
                ..default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            ..default()
        },
        Structure {
            tier: 2,
            structure_type: StructureType::Turret,
            durability: TURRET_HP,
            grid_pos,
        },
        Health::new(TURRET_HP),
        TurretMarker,
        TurretState {
            emp_timer: 0.0,
            fire_timer: 0.0,
            range: TURRET_RANGE,
            damage: TURRET_DAMAGE,
            fire_rate: TURRET_FIRE_RATE,
        },
    )).id();

    // Barrel: child entity yang berputar ke target
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_TURRET_BARREL,
                custom_size: Some(Vec2::new(6.0, 18.0)), // lonjong = barrel
                ..default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y + 10.0, 1.1),
            ..default()
        },
        TurretBarrel { turret_entity },
    ));
}

// ---------------------------------------------------------------------------
// S1-07: Spawn Farm
// ---------------------------------------------------------------------------

fn spawn_farm(commands: &mut Commands, world_pos: Vec2, grid_pos: IVec2) {
    // Farm berukuran 2×1 tile (64×32)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_FARM,
                custom_size: Some(Vec2::new(TILE_SIZE * 2.0 - 2.0, TILE_SIZE - 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            ..default()
        },
        Structure {
            tier: 1,
            structure_type: StructureType::Farm,
            durability: FARM_HP,
            grid_pos,
        },
        Health::new(FARM_HP),
        FarmMarker,
    ));
}

// ---------------------------------------------------------------------------
// House (untuk S2-08)
// ---------------------------------------------------------------------------

fn spawn_house(commands: &mut Commands, world_pos: Vec2, grid_pos: IVec2, colony: &mut ColonyState) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_HOUSE,
                custom_size: Some(Vec2::splat(TILE_SIZE * 2.0 - 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
            ..default()
        },
        Structure {
            tier: 1,
            structure_type: StructureType::House,
            durability: HOUSE_HP,
            grid_pos,
        },
        Health::new(HOUSE_HP),
        HouseMarker,
    ));
    // Tambah kapasitas populasi +4 per house
    colony.max_population += 4;
}

// ---------------------------------------------------------------------------
// S1-06: Turret AI — auto-target, fire
// ---------------------------------------------------------------------------

fn turret_ai(
    mut commands: Commands,
    // ParamSet: isolasi semua Query yang akses Transform secara bersamaan.
    // Tanpa ini Bevy panic B0001 karena barrel_q minta &mut Transform
    // sementara turret_q dan enemy_q juga akses &Transform di frame yang sama.
    mut tf_set: ParamSet<(
        // p0 — turret: baca posisi + state
        Query<(Entity, &Transform, &mut TurretState), With<TurretMarker>>,
        // p1 — barrel: tulis rotasi + posisi
        Query<(&TurretBarrel, &mut Transform), Without<TurretMarker>>,
        // p2 — enemy: baca posisi untuk targeting
        Query<(Entity, &Transform), With<Enemy>>,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // Kumpulkan data turret dulu sebelum akses query lain —
    // ParamSet tidak boleh dua p aktif sekaligus.
    struct TurretData {
        entity:    Entity,
        pos:       Vec2,
        fire_ready: bool,
        fire_rate: f32,
        range:     f32,
        damage:    f32,
    }

    // Pass 1: baca turret state, update timer
    let mut turrets: Vec<TurretData> = Vec::new();
    for (ent, tf, mut state) in &mut tf_set.p0() {
        if state.emp_timer > 0.0 {
            state.emp_timer -= dt;
            continue;
        }
        state.fire_timer -= dt;
        turrets.push(TurretData {
            entity:    ent,
            pos:       tf.translation.truncate(),
            fire_ready: state.fire_timer <= 0.0,
            fire_rate: state.fire_rate,
            range:     state.range,
            damage:    state.damage,
        });
        // Reset fire timer langsung di state (borrow sudah selesai di akhir loop)
    }

    // Reset fire_timer untuk turret yang fire_ready — harus re-borrow p0
    for data in &turrets {
        if data.fire_ready {
            if let Ok((_, _, mut state)) = tf_set.p0().get_mut(data.entity) {
                state.fire_timer = data.fire_rate;
            }
        }
    }

    // Pass 2: kumpulkan posisi semua enemy (p2, read-only)
    let enemies: Vec<(Entity, Vec2)> = tf_set.p2()
        .iter()
        .map(|(e, tf)| (e, tf.translation.truncate()))
        .collect();

    // Pass 3: update barrel rotation (p1, write)
    // Sekaligus kumpulkan spawn data untuk projectile
    struct FireData {
        turret_pos: Vec2,
        angle:      f32,
        target:     Entity,
        damage:     f32,
    }
    let mut to_fire: Vec<FireData> = Vec::new();

    for data in &turrets {
        // Cari enemy terdekat dalam range
        let closest = enemies.iter()
            .filter(|(_, epos)| data.pos.distance(*epos) <= data.range)
            .min_by(|(_, a), (_, b)| {
                data.pos.distance(*a)
                    .partial_cmp(&data.pos.distance(*b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        let Some((target_ent, target_pos)) = closest else { continue };

        let dir = (*target_pos - data.pos).normalize_or_zero();
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;

        // Putar barrel yang milik turret ini
        for (barrel, mut barrel_tf) in &mut tf_set.p1() {
            if barrel.turret_entity == data.entity {
                barrel_tf.rotation = Quat::from_rotation_z(angle);
                barrel_tf.translation.x = data.pos.x + dir.x * 8.0;
                barrel_tf.translation.y = data.pos.y + dir.y * 8.0;
            }
        }

        if data.fire_ready {
            to_fire.push(FireData {
                turret_pos: data.pos,
                angle,
                target: *target_ent,
                damage: data.damage,
            });
        }
    }

    // Pass 4: spawn projectile (Commands, tidak butuh Query lagi)
    for fd in to_fire {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: COLOR_PROJECTILE,
                    custom_size: Some(Vec2::new(4.0, 8.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(fd.turret_pos.x, fd.turret_pos.y, 1.5),
                    rotation: Quat::from_rotation_z(fd.angle),
                    ..default()
                },
                ..default()
            },
            TurretProjectile {
                target: fd.target,
                damage: fd.damage,
                speed: TURRET_PROJECTILE_SPEED,
            },
        ));
    }
}

// ---------------------------------------------------------------------------
// Turret Projectile Movement
// ---------------------------------------------------------------------------

fn turret_projectile_move(
    mut commands: Commands,
    mut proj_q: Query<(Entity, &mut Transform, &TurretProjectile)>,
    target_q: Query<&Transform, (With<Enemy>, Without<TurretProjectile>)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (proj_ent, mut proj_tf, proj) in &mut proj_q {
        // Cek apakah target masih ada
        let Ok(target_tf) = target_q.get(proj.target) else {
            // Target sudah mati — despawn projectile
            commands.entity(proj_ent).despawn();
            continue;
        };

        let proj_pos = proj_tf.translation.truncate();
        let target_pos = target_tf.translation.truncate();
        let to_target = target_pos - proj_pos;
        let dist = to_target.length();

        if dist < 8.0 {
            // Hit! damage enemy
            damage_events.send(DamageEvent {
                target: proj.target,
                amount: proj.damage,
                from_turret: true,
                from_npc: false,
            });
            commands.entity(proj_ent).despawn();
        } else {
            // Gerak menuju target (homing sederhana)
            let dir = to_target.normalize();
            let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
            proj_tf.translation.x += dir.x * proj.speed * dt;
            proj_tf.translation.y += dir.y * proj.speed * dt;
            proj_tf.rotation = Quat::from_rotation_z(angle);
        }
    }
}

// ---------------------------------------------------------------------------
// S1-07: Farm Food Production
// ---------------------------------------------------------------------------

fn farm_produce_food(
    farm_q: Query<&Health, With<FarmMarker>>,
    mut colony: ResMut<ColonyState>,
    mut resources: ResMut<PlayerResources>,
    mut phase_events: EventReader<crate::components::PhaseChanged>,
) {
    for ev in phase_events.read() {
        // Produksi food setiap transisi ke Day (pagi hari)
        if ev.new_phase == Phase::Day {
            let mut total_food = 0u32;

            for health in &farm_q {
                // Farm yang rusak berat (HP < 30%) tidak produksi
                if health.fraction() >= 0.30 {
                    total_food += FARM_FOOD_PER_DAY;
                } else {
                    total_food += 1; // produksi minimal kalau hampir hancur
                }
            }

            if total_food > 0 {
                resources.food += total_food;
                colony.food += total_food;
                bevy::log::info!("[Farm] Produksi {total_food} food di hari {}", ev.day);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Visual Damage — warna berubah sesuai HP
// ---------------------------------------------------------------------------

fn structure_damage_visual(
    mut structure_q: Query<(&Health, &Structure, &mut Sprite)>,
) {
    for (health, structure, mut sprite) in &mut structure_q {
        let hp_frac = health.fraction();
        // Gelapkan warna proporsional dengan kerusakan
        let base_color = match structure.structure_type {
            StructureType::Wall    => COLOR_WALL,
            StructureType::Turret  => COLOR_TURRET_BASE,
            StructureType::Farm    => COLOR_FARM,
            StructureType::House   => COLOR_HOUSE,
            _ => continue,
        };

        // Lerp ke merah gelap saat HP rendah
        let damage_tint = if hp_frac < 0.3 {
            Color::srgb(0.5, 0.1, 0.1) // kritis: merah gelap
        } else if hp_frac < 0.6 {
            Color::srgb(0.4, 0.3, 0.15) // rusak: oranye gelap
        } else {
            base_color
        };

        sprite.color = damage_tint;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Snap posisi world ke grid 32px
fn snap_to_grid(pos: Vec2) -> Vec2 {
    let half = TILE_SIZE / 2.0;
    Vec2::new(
        (pos.x / TILE_SIZE).floor() * TILE_SIZE + half,
        (pos.y / TILE_SIZE).floor() * TILE_SIZE + half,
    )
}

/// World pos → grid cell index (IVec2)
fn world_to_grid(world_pos: Vec2) -> IVec2 {
    IVec2::new(
        (world_pos.x / TILE_SIZE).floor() as i32,
        (world_pos.y / TILE_SIZE).floor() as i32,
    )
}

/// Ukuran sprite per jenis struktur
fn structure_size(sel: BuildSelection) -> Vec2 {
    match sel {
        BuildSelection::Wall   => Vec2::splat(TILE_SIZE - 2.0),
        BuildSelection::Turret => Vec2::splat(TILE_SIZE - 4.0),
        BuildSelection::Farm   => Vec2::new(TILE_SIZE * 2.0 - 2.0, TILE_SIZE - 2.0),
        BuildSelection::House  => Vec2::splat(TILE_SIZE * 2.0 - 2.0),
    }
}

// BuildMode dan BuildSelection sudah pub di definisinya — tidak perlu re-export
