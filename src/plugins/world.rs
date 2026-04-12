// Void Architect — plugins/world.rs
// Tile map procedural generation (Ashlands biome), camera follow,
// dan spawning resource node di peta. [S0-03, S0-07]

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::components::*;
use crate::GameState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: i32 = 40;   // tiles
pub const MAP_HEIGHT: i32 = 30;  // tiles

// Warna palette Ashlands (MVP geometric rendering)
const COLOR_TILE_BASE: Color    = Color::rgb(0.15, 0.12, 0.10);
const COLOR_TILE_VARIANT_A: Color = Color::rgb(0.18, 0.14, 0.11);
const COLOR_TILE_VARIANT_B: Color = Color::rgb(0.12, 0.10, 0.08);
const COLOR_TILE_ROCK: Color    = Color::rgb(0.25, 0.22, 0.20);
const COLOR_TILE_ASH: Color     = Color::rgb(0.20, 0.20, 0.20);

// Warna resource node
const COLOR_NODE_STONE: Color   = Color::rgb(0.55, 0.55, 0.60);
const COLOR_NODE_SCRAP: Color   = Color::rgb(0.70, 0.50, 0.20);
const COLOR_NODE_CRYSTAL: Color = Color::rgb(0.40, 0.20, 0.80);
const COLOR_NODE_FOOD: Color    = Color::rgb(0.30, 0.65, 0.25);

const RESOURCE_NODE_COUNT: usize = 20;
const CRYSTAL_NODE_COUNT: usize = 3;

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

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Seed untuk procedural generation — bisa di-set dari meta unlock.
#[derive(Resource, Debug, Clone)]
pub struct MapSeed(pub u64);

impl Default for MapSeed {
    fn default() -> Self {
        Self(42) // default seed; di-randomize saat run dimulai
    }
}

// ---------------------------------------------------------------------------
// Tile Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum TileVariant {
    Base,
    VariantA,
    VariantB,
    RockPatch,
    AshSpot,
}

impl TileVariant {
    fn color(&self) -> Color {
        match self {
            TileVariant::Base     => COLOR_TILE_BASE,
            TileVariant::VariantA => COLOR_TILE_VARIANT_A,
            TileVariant::VariantB => COLOR_TILE_VARIANT_B,
            TileVariant::RockPatch => COLOR_TILE_ROCK,
            TileVariant::AshSpot  => COLOR_TILE_ASH,
        }
    }

    fn from_noise(value: f64) -> Self {
        match value {
            v if v > 0.6  => TileVariant::RockPatch,
            v if v > 0.3  => TileVariant::VariantA,
            v if v > 0.0  => TileVariant::Base,
            v if v > -0.3 => TileVariant::VariantB,
            _             => TileVariant::AshSpot,
        }
    }
}

/// Marker komponen untuk tile entity.
#[derive(Component)]
pub struct Tile {
    pub grid_x: i32,
    pub grid_y: i32,
    pub variant: TileVariant,
}

// ---------------------------------------------------------------------------
// Tilemap Generation (S0-03)
// ---------------------------------------------------------------------------

/// Spawn seluruh tilemap Ashlands menggunakan Perlin noise.
fn spawn_tilemap(
    mut commands: Commands,
    seed: Res<MapSeed>,
) {
    let perlin = Perlin::new(seed.0 as u32);
    let noise_scale = 0.15; // ukuran fitur noise — lebih kecil = patch lebih besar

    let map_offset = Vec2::new(
        -(MAP_WIDTH as f32 * TILE_SIZE) / 2.0,
        -(MAP_HEIGHT as f32 * TILE_SIZE) / 2.0,
    );

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let nx = x as f64 * noise_scale;
            let ny = y as f64 * noise_scale;
            let noise_value = perlin.get([nx, ny]);

            let variant = TileVariant::from_noise(noise_value);
            let world_pos = Vec2::new(
                map_offset.x + x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                map_offset.y + y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: variant.color(),
                        custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), // 1px gap = grid lines
                        ..default()
                    },
                    transform: Transform::from_xyz(world_pos.x, world_pos.y, -1.0), // z = -1 = background
                    ..default()
                },
                Tile { grid_x: x, grid_y: y, variant },
            ));
        }
    }

    // Spawn border gelap di sekeliling map sebagai batas visual
    spawn_map_border(&mut commands, map_offset);
}

/// Spawn border gelap sebagai batas visual peta.
fn spawn_map_border(commands: &mut Commands, map_offset: Vec2) {
    let border_thickness = TILE_SIZE;
    let map_w = MAP_WIDTH as f32 * TILE_SIZE;
    let map_h = MAP_HEIGHT as f32 * TILE_SIZE;
    let border_color = Color::rgb(0.05, 0.04, 0.03);

    // Top, Bottom, Left, Right
    let borders = [
        (Vec2::new(map_offset.x + map_w/2.0, map_offset.y + map_h + border_thickness/2.0),
         Vec2::new(map_w + border_thickness*2.0, border_thickness)),
        (Vec2::new(map_offset.x + map_w/2.0, map_offset.y - border_thickness/2.0),
         Vec2::new(map_w + border_thickness*2.0, border_thickness)),
        (Vec2::new(map_offset.x - border_thickness/2.0, map_offset.y + map_h/2.0),
         Vec2::new(border_thickness, map_h)),
        (Vec2::new(map_offset.x + map_w + border_thickness/2.0, map_offset.y + map_h/2.0),
         Vec2::new(border_thickness, map_h)),
    ];

    for (pos, size) in borders {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: border_color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, -0.5),
            ..default()
        });
    }
}

// ---------------------------------------------------------------------------
// Resource Node Spawning (S0-07)
// ---------------------------------------------------------------------------

/// Scatter resource nodes di map secara random, hindari center (area Void Core).
fn spawn_resource_nodes(
    mut commands: Commands,
    seed: Res<MapSeed>,
) {
    let mut rng = StdRng::seed_from_u64(seed.0 + 1000);
    let map_half_w = (MAP_WIDTH as f32 * TILE_SIZE) / 2.0;
    let map_half_h = (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0;
    let void_core_radius = 120.0; // jarak aman dari center

    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < RESOURCE_NODE_COUNT && attempts < 500 {
        attempts += 1;

        let x = rng.gen_range(-map_half_w + TILE_SIZE..map_half_w - TILE_SIZE);
        let y = rng.gen_range(-map_half_h + TILE_SIZE..map_half_h - TILE_SIZE);
        let pos = Vec2::new(x, y);

        // Hindari center (Void Core area)
        if pos.length() < void_core_radius {
            continue;
        }

        // Roll untuk tipe resource
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
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(14.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            node,
            Position(pos),
        ));

        spawned += 1;
    }

    // Spawn void crystal nodes (langka)
    for _ in 0..CRYSTAL_NODE_COUNT {
        let x = rng.gen_range(-map_half_w + TILE_SIZE..map_half_w - TILE_SIZE);
        let y = rng.gen_range(-map_half_h + TILE_SIZE..map_half_h - TILE_SIZE);

        if Vec2::new(x, y).length() < void_core_radius {
            continue;
        }

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: COLOR_NODE_CRYSTAL,
                    custom_size: Some(Vec2::splat(12.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            },
            ResourceNode { stone: 0, scrap: 0, void_crystal: rng.gen_range(1..4), food: 0 },
            Position(Vec2::new(x, y)),
        ));
    }
}

// ---------------------------------------------------------------------------
// Void Core (pusat map)
// ---------------------------------------------------------------------------

/// Spawn Void Core entity di tengah map.
fn spawn_void_core(
    mut commands: Commands,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.6, 0.1, 0.9),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Health::new(500.0),
        VoidCore,
        Position(Vec2::ZERO),
    ));
}

// ---------------------------------------------------------------------------
// Camera Follow (S0-03)
// ---------------------------------------------------------------------------

/// Sistem yang membuat kamera mengikuti player dengan smooth lerp.
fn camera_follow_player(
    player_q: Query<&Transform, (With<crate::plugins::player::PlayerMarker>, Without<Camera>)>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let Ok(mut cam_tf) = camera_q.get_single_mut() else { return };

    let follow_speed = 5.0;
    let target = player_tf.translation;
    cam_tf.translation = cam_tf.translation.lerp(target, follow_speed * time.delta_seconds());
    cam_tf.translation.z = 999.9; // kamera selalu di atas semua layer
}

// ---------------------------------------------------------------------------
// Resource Node Collection (S0-07)
// ---------------------------------------------------------------------------

const COLLECT_RADIUS: f32 = 20.0;

/// Player menyentuh resource node → collect otomatis, despawn node.
fn collect_resource_nodes(
    mut commands: Commands,
    player_q: Query<&Transform, With<crate::plugins::player::PlayerMarker>>,
    node_q: Query<(Entity, &Transform, &ResourceNode)>,
    mut resources: ResMut<PlayerResources>,
    mut colony: ResMut<ColonyState>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, node_tf, node) in &node_q {
        let node_pos = node_tf.translation.truncate();
        if player_pos.distance(node_pos) < COLLECT_RADIUS {
            resources.stone += node.stone;
            resources.scrap += node.scrap;
            resources.void_crystal += node.void_crystal;
            let food_gained = node.food;
            resources.food += food_gained;
            colony.food += food_gained;

            commands.entity(entity).despawn();
        }
    }
}
