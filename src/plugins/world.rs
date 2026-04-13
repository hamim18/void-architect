// Void Architect — plugins/world.rs
// Tile map, camera follow, resource node spawning & collection.
// [S0-03, S0-07]
//
// FIX-01: Resource respawn tiap hari (PhaseChanged(Day))
// FIX-06: Pickup radius lebih besar (magnet 48px), XP drop collected on pickup

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::components::*;
use crate::GameState;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: i32 = 40;
pub const MAP_HEIGHT: i32 = 30;

// Map half-extents (dipakai player.rs & colony.rs untuk clamping)
pub const MAP_HALF_W: f32 = MAP_WIDTH as f32 * TILE_SIZE / 2.0;
pub const MAP_HALF_H: f32 = MAP_HEIGHT as f32 * TILE_SIZE / 2.0;

const COLOR_TILE_BASE: Color      = Color::srgb(0.15, 0.12, 0.10);
const COLOR_TILE_VARIANT_A: Color = Color::srgb(0.18, 0.14, 0.11);
const COLOR_TILE_VARIANT_B: Color = Color::srgb(0.12, 0.10, 0.08);
const COLOR_TILE_ROCK: Color      = Color::srgb(0.25, 0.22, 0.20);
const COLOR_TILE_ASH: Color       = Color::srgb(0.20, 0.20, 0.20);
const COLOR_NODE_STONE: Color     = Color::srgb(0.55, 0.55, 0.60);
const COLOR_NODE_SCRAP: Color     = Color::srgb(0.70, 0.50, 0.20);
const COLOR_NODE_CRYSTAL: Color   = Color::srgb(0.40, 0.20, 0.80);
const COLOR_NODE_FOOD: Color      = Color::srgb(0.30, 0.65, 0.25);
pub const COLOR_XP_DROP: Color    = Color::srgb(0.85, 0.90, 0.20); // kuning terang

const RESOURCE_NODE_COUNT: usize = 20;
const CRYSTAL_NODE_COUNT: usize  = 3;

/// FIX-06: Radius magnet — pickup terpicu saat dalam radius ini
pub const COLLECT_RADIUS: f32 = 48.0;

const VOID_CORE_CLEAR_RADIUS: f32 = 120.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker untuk tile entity
#[derive(Component)]
pub struct Tile;

/// FIX-01: Marker resource node agar bisa di-despawn & re-spawn per hari
#[derive(Component)]
pub struct WorldResourceNode;

/// FIX-05: XP drop yang jatuh dari enemy mati — harus dipungut player
#[derive(Component, Debug, Clone, Copy)]
pub struct XpDrop {
    pub amount: u32,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MapSeed::default())
            .add_systems(OnEnter(GameState::InRun), (
                spawn_tilemap,
                spawn_resource_nodes,
                spawn_void_core_world, // Void Core di world (visual only — HP di progression.rs)
                spawn_map_boundaries,  // FIX-03: invisible wall di tepi map
            ).chain())
            .add_systems(Update, (
                camera_follow_player,
                collect_resource_nodes, // FIX-06: magnet radius
                collect_xp_drops,       // FIX-05 + FIX-06: XP pickup magnet
                respawn_resource_nodes, // FIX-01: respawn tiap hari
            ).run_if(in_state(GameState::InRun)));
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MapSeed(pub u64);
impl Default for MapSeed { fn default() -> Self { Self(42) } }

// ---------------------------------------------------------------------------
// Tilemap
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum TileVariant { Base, VariantA, VariantB, RockPatch, AshSpot }

impl TileVariant {
    fn color(self) -> Color {
        match self {
            TileVariant::Base      => COLOR_TILE_BASE,
            TileVariant::VariantA  => COLOR_TILE_VARIANT_A,
            TileVariant::VariantB  => COLOR_TILE_VARIANT_B,
            TileVariant::RockPatch => COLOR_TILE_ROCK,
            TileVariant::AshSpot   => COLOR_TILE_ASH,
        }
    }
    fn from_noise(v: f64) -> Self {
        if v > 0.6       { TileVariant::RockPatch }
        else if v > 0.3  { TileVariant::VariantA }
        else if v > 0.0  { TileVariant::Base }
        else if v > -0.3 { TileVariant::VariantB }
        else             { TileVariant::AshSpot }
    }
}

fn spawn_tilemap(mut commands: Commands, seed: Res<MapSeed>) {
    let perlin = Perlin::new(seed.0 as u32);
    let scale = 0.15_f64;
    let ox = -MAP_HALF_W;
    let oy = -MAP_HALF_H;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let variant = TileVariant::from_noise(
                perlin.get([x as f64 * scale, y as f64 * scale])
            );
            let wx = ox + x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let wy = oy + y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite { color: variant.color(), custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), ..default() },
                    transform: Transform::from_xyz(wx, wy, -10.0),
                    ..default()
                },
                Tile,
            ));
        }
    }

    // Border gelap
    let border = TILE_SIZE;
    let map_w = MAP_WIDTH as f32 * TILE_SIZE;
    let map_h = MAP_HEIGHT as f32 * TILE_SIZE;
    let bc = Color::srgb(0.05, 0.04, 0.03);
    for (pos, size) in [
        (Vec2::new(ox + map_w/2.0, oy + map_h + border/2.0), Vec2::new(map_w + border*2.0, border)),
        (Vec2::new(ox + map_w/2.0, oy - border/2.0),          Vec2::new(map_w + border*2.0, border)),
        (Vec2::new(ox - border/2.0, oy + map_h/2.0),          Vec2::new(border, map_h)),
        (Vec2::new(ox + map_w + border/2.0, oy + map_h/2.0),  Vec2::new(border, map_h)),
    ] {
        commands.spawn(SpriteBundle {
            sprite: Sprite { color: bc, custom_size: Some(size), ..default() },
            transform: Transform::from_xyz(pos.x, pos.y, -9.0),
            ..default()
        });
    }
}

// ---------------------------------------------------------------------------
// FIX-01: Resource Nodes — spawn + respawn tiap Day
// ---------------------------------------------------------------------------

fn spawn_resource_nodes(mut commands: Commands, seed: Res<MapSeed>) {
    do_spawn_nodes(&mut commands, seed.0, 0);
}

/// FIX-01: Tiap kali Day dimulai, hapus node lama dan spawn ulang.
/// Seed dibuat dari hari ke-n agar posisi berbeda tiap hari.
fn respawn_resource_nodes(
    mut commands: Commands,
    mut phase_events: EventReader<PhaseChanged>,
    old_nodes: Query<Entity, With<WorldResourceNode>>,
    seed: Res<MapSeed>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != Phase::Day { continue; }
        // Hapus semua node yang tersisa
        for e in &old_nodes { commands.entity(e).despawn(); }
        // Spawn ulang dengan seed baru (seed ^ day untuk variasi posisi)
        do_spawn_nodes(&mut commands, seed.0, ev.day as u64);
        bevy::log::info!("[World] Resource nodes respawned untuk Day {}", ev.day);
    }
}

fn do_spawn_nodes(commands: &mut Commands, base_seed: u64, day: u64) {
    let mut rng = StdRng::seed_from_u64(base_seed.wrapping_add(day * 999_983));

    let mut spawned = 0;
    let mut attempts = 0;
    while spawned < RESOURCE_NODE_COUNT && attempts < 500 {
        attempts += 1;
        let x = rng.gen_range(-MAP_HALF_W + TILE_SIZE..MAP_HALF_W - TILE_SIZE);
        let y = rng.gen_range(-MAP_HALF_H + TILE_SIZE..MAP_HALF_H - TILE_SIZE);
        if Vec2::new(x, y).length() < VOID_CORE_CLEAR_RADIUS { continue; }

        let roll: f32 = rng.gen();
        let (node, color) = if roll < 0.50 {
            (ResourceNode { stone: rng.gen_range(30..80), scrap: 0, void_crystal: 0, food: 0 }, COLOR_NODE_STONE)
        } else if roll < 0.80 {
            (ResourceNode { stone: 0, scrap: rng.gen_range(15..45), void_crystal: 0, food: 0 }, COLOR_NODE_SCRAP)
        } else {
            (ResourceNode { stone: 0, scrap: 0, void_crystal: 0, food: rng.gen_range(3..10) }, COLOR_NODE_FOOD)
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite { color, custom_size: Some(Vec2::splat(14.0)), ..default() },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            node,
            WorldResourceNode,
        ));
        spawned += 1;
    }

    for _ in 0..CRYSTAL_NODE_COUNT {
        let x = rng.gen_range(-MAP_HALF_W + TILE_SIZE..MAP_HALF_W - TILE_SIZE);
        let y = rng.gen_range(-MAP_HALF_H + TILE_SIZE..MAP_HALF_H - TILE_SIZE);
        if Vec2::new(x, y).length() < VOID_CORE_CLEAR_RADIUS { continue; }
        commands.spawn((
            SpriteBundle {
                sprite: Sprite { color: COLOR_NODE_CRYSTAL, custom_size: Some(Vec2::splat(12.0)), ..default() },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            ResourceNode { stone: 0, scrap: 0, void_crystal: rng.gen_range(1..4), food: 0 },
            WorldResourceNode,
        ));
    }
}

// ---------------------------------------------------------------------------
// Void Core (world visual — HP logic di progression.rs)
// ---------------------------------------------------------------------------

fn spawn_void_core_world(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.6, 0.1, 0.9),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Health::new(500.0),
        VoidCore,
    ));
}

// ---------------------------------------------------------------------------
// FIX-06: Collect resource nodes — magnet radius
// ---------------------------------------------------------------------------

fn collect_resource_nodes(
    mut commands: Commands,
    player_q: Query<&Transform, With<crate::plugins::player::PlayerMarker>>,
    node_q: Query<(Entity, &Transform, &ResourceNode)>,
    mut resources: ResMut<PlayerResources>,
    mut colony: ResMut<ColonyState>,
) {
    let Ok(ptf) = player_q.get_single() else { return };
    let ppos = ptf.translation.truncate();

    for (entity, ntf, node) in &node_q {
        // FIX-06: radius diperbesar ke 48px — terasa seperti magnet
        if ppos.distance(ntf.translation.truncate()) < COLLECT_RADIUS {
            resources.stone        += node.stone;
            resources.scrap        += node.scrap;
            resources.void_crystal += node.void_crystal;
            resources.food         += node.food;
            colony.food            += node.food;
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// FIX-05 + FIX-06: XP Drop — spawn dari enemy mati, pickup by player
// ---------------------------------------------------------------------------

/// Spawn XP drop di posisi enemy mati (dipanggil dari combat.rs).
pub fn spawn_xp_drop(commands: &mut Commands, pos: Vec2, amount: u32) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_XP_DROP,
                custom_size: Some(Vec2::splat(8.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.8),
            ..default()
        },
        XpDrop { amount },
    ));
}

/// FIX-05 + FIX-06: Kumpulkan XP drop saat player mendekati (magnet 48px).
/// EXP ditambahkan ke player.exp langsung.
fn collect_xp_drops(
    mut commands: Commands,
    mut player_q: Query<
        (&Transform, &mut Player, Option<&crate::plugins::progression::KillChainState>),
        With<crate::plugins::player::PlayerMarker>,
    >,
    xp_q: Query<(Entity, &Transform, &XpDrop)>,
) {
    let Ok((ptf, mut player, kc_state)) = player_q.get_single_mut() else { return };
    let ppos = ptf.translation.truncate();
    // FIX-05: KillChain multiplier diterapkan di sini saat pickup
    let kc_mult: u32 = if kc_state.map_or(false, |s: &crate::plugins::progression::KillChainState| s.is_active) { 2 } else { 1 };

    for (entity, xtf, xp) in &xp_q {
        if ppos.distance(xtf.translation.truncate()) < COLLECT_RADIUS {
            player.exp += xp.amount * kc_mult;
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Camera Follow
// ---------------------------------------------------------------------------

fn camera_follow_player(
    player_q: Query<&Transform, (With<crate::plugins::player::PlayerMarker>, Without<Camera>)>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let Ok(ptf) = player_q.get_single() else { return };
    let Ok(mut ctf) = camera_q.get_single_mut() else { return };
    let target_xy = ptf.translation.truncate();
    let current_xy = ctf.translation.truncate();
    let new_xy = current_xy.lerp(target_xy, 5.0 * time.delta_seconds());
    ctf.translation.x = new_xy.x;
    ctf.translation.y = new_xy.y;
    // z TIDAK disentuh (ADR-08)
}


// ---------------------------------------------------------------------------
// FIX-03: Invisible wall colliders di tepi map — block NPC & enemy keluar map
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct MapBoundary;

fn spawn_map_boundaries(mut commands: Commands) {
    use bevy_rapier2d::prelude::*;

    let hw = MAP_HALF_W;
    let hh = MAP_HALF_H;
    let thickness = 20.0;

    // Top, Bottom, Left, Right
    for (x, y, w, h) in [
        (0.0,  hh + thickness / 2.0, hw * 2.0 + thickness * 2.0, thickness), // top
        (0.0, -hh - thickness / 2.0, hw * 2.0 + thickness * 2.0, thickness), // bottom
        (-hw - thickness / 2.0, 0.0, thickness, hh * 2.0),                    // left
        ( hw + thickness / 2.0, 0.0, thickness, hh * 2.0),                    // right
    ] {
        commands.spawn((
            TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
            RigidBody::Fixed,
            Collider::cuboid(w / 2.0, h / 2.0),
            MapBoundary,
        ));
    }
}
