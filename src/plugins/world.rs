// Void Architect — plugins/world.rs
// Tile map procedural generation (Ashlands), camera follow,
// resource node spawning & collection. [S0-03, S0-07]

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::components::*;
use crate::GameState;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: i32 = 40;
pub const MAP_HEIGHT: i32 = 30;

const COLOR_TILE_BASE: Color      = Color::srgb(0.15, 0.12, 0.10);
const COLOR_TILE_VARIANT_A: Color = Color::srgb(0.18, 0.14, 0.11);
const COLOR_TILE_VARIANT_B: Color = Color::srgb(0.12, 0.10, 0.08);
const COLOR_TILE_ROCK: Color      = Color::srgb(0.25, 0.22, 0.20);
const COLOR_TILE_ASH: Color       = Color::srgb(0.20, 0.20, 0.20);
const COLOR_NODE_STONE: Color     = Color::srgb(0.55, 0.55, 0.60);
const COLOR_NODE_SCRAP: Color     = Color::srgb(0.70, 0.50, 0.20);
const COLOR_NODE_CRYSTAL: Color   = Color::srgb(0.40, 0.20, 0.80);
const COLOR_NODE_FOOD: Color      = Color::srgb(0.30, 0.65, 0.25);

const RESOURCE_NODE_COUNT: usize = 20;
const CRYSTAL_NODE_COUNT: usize = 3;
const COLLECT_RADIUS: f32 = 20.0;
const VOID_CORE_CLEAR_RADIUS: f32 = 120.0;

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
                spawn_void_core,
            ).chain())
            .add_systems(Update, (
                camera_follow_player,
                collect_resource_nodes,
            ).run_if(in_state(GameState::InRun)));
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MapSeed(pub u64);
impl Default for MapSeed {
    fn default() -> Self { Self(42) }
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum TileVariant {
    Base, VariantA, VariantB, RockPatch, AshSpot,
}

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

/// Marker untuk tile entity — dipakai cleanup system nanti.
#[derive(Component)]
pub struct Tile;

// ---------------------------------------------------------------------------
// Tilemap Spawn
// ---------------------------------------------------------------------------

fn spawn_tilemap(mut commands: Commands, seed: Res<MapSeed>) {
    let perlin = Perlin::new(seed.0 as u32);
    let scale = 0.15_f64;

    // Offset agar map di-center di origin
    let ox = -(MAP_WIDTH as f32 * TILE_SIZE) / 2.0;
    let oy = -(MAP_HEIGHT as f32 * TILE_SIZE) / 2.0;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let variant = TileVariant::from_noise(
                perlin.get([x as f64 * scale, y as f64 * scale])
            );
            let wx = ox + x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let wy = oy + y as f32 * TILE_SIZE + TILE_SIZE / 2.0;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: variant.color(),
                        custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                        ..default()
                    },
                    // z = -10.0: jauh di belakang semua gameplay entity
                    // Camera default projection melihat dari z=1000 ke z=-1000,
                    // jadi z=-10 masih aman.
                    transform: Transform::from_xyz(wx, wy, -10.0),
                    ..default()
                },
                Tile,
            ));
        }
    }

    // Border gelap sekeliling map
    let border = TILE_SIZE;
    let map_w  = MAP_WIDTH as f32 * TILE_SIZE;
    let map_h  = MAP_HEIGHT as f32 * TILE_SIZE;
    let bc     = Color::srgb(0.05, 0.04, 0.03);

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
// Resource Nodes
// ---------------------------------------------------------------------------

fn spawn_resource_nodes(mut commands: Commands, seed: Res<MapSeed>) {
    let mut rng = StdRng::seed_from_u64(seed.0 + 1000);
    let hw = (MAP_WIDTH as f32 * TILE_SIZE) / 2.0;
    let hh = (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0;

    let mut spawned = 0;
    let mut attempts = 0;
    while spawned < RESOURCE_NODE_COUNT && attempts < 500 {
        attempts += 1;
        let x = rng.gen_range(-hw + TILE_SIZE..hw - TILE_SIZE);
        let y = rng.gen_range(-hh + TILE_SIZE..hh - TILE_SIZE);
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
        ));
        spawned += 1;
    }

    for _ in 0..CRYSTAL_NODE_COUNT {
        let x = rng.gen_range(-hw + TILE_SIZE..hw - TILE_SIZE);
        let y = rng.gen_range(-hh + TILE_SIZE..hh - TILE_SIZE);
        if Vec2::new(x, y).length() < VOID_CORE_CLEAR_RADIUS { continue; }
        commands.spawn((
            SpriteBundle {
                sprite: Sprite { color: COLOR_NODE_CRYSTAL, custom_size: Some(Vec2::splat(12.0)), ..default() },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            ResourceNode { stone: 0, scrap: 0, void_crystal: rng.gen_range(1..4), food: 0 },
        ));
    }
}

// ---------------------------------------------------------------------------
// Void Core
// ---------------------------------------------------------------------------

fn spawn_void_core(mut commands: Commands) {
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
// Camera Follow
// ---------------------------------------------------------------------------

fn camera_follow_player(
    player_q: Query<&Transform, (With<crate::plugins::player::PlayerMarker>, Without<Camera>)>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let Ok(ptf) = player_q.get_single() else { return };
    let Ok(mut ctf) = camera_q.get_single_mut() else { return };

    // Lerp hanya XY — Z kamera JANGAN diubah.
    // Camera2dBundle default z = 999.9, biarkan di sana.
    let target_xy = ptf.translation.truncate();
    let current_xy = ctf.translation.truncate();
    let new_xy = current_xy.lerp(target_xy, 5.0 * time.delta_seconds());

    ctf.translation.x = new_xy.x;
    ctf.translation.y = new_xy.y;
    // ctf.translation.z TIDAK disentuh
}

// ---------------------------------------------------------------------------
// Resource Collection
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
        if ppos.distance(ntf.translation.truncate()) < COLLECT_RADIUS {
            resources.stone       += node.stone;
            resources.scrap       += node.scrap;
            resources.void_crystal += node.void_crystal;
            resources.food        += node.food;
            colony.food           += node.food;
            commands.entity(entity).despawn();
        }
    }
}
