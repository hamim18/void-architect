// Void Architect — plugins/player.rs
// Player controller: WASD movement, mouse facing, velocity damping.
// Physics via bevy_rapier2d. [S0-04]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::components::*;
use crate::GameState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PLAYER_SPEED: f32 = 180.0;         // px/s
const PLAYER_DAMPING: f32 = 10.0;        // linear damping
const PLAYER_COLLIDER_RADIUS: f32 = 10.0;

// Warna player (MVP geometric: cyan triangle)
const COLOR_PLAYER: Color = Color::rgb(0.0, 0.85, 0.85);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            // Debug colliders: aktifkan saat development, matikan di release
            // .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(OnEnter(GameState::InRun), spawn_player)
            .add_systems(Update, (
                player_movement,
                player_mouse_facing,
                cooldown_tick,
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Marker
// ---------------------------------------------------------------------------

/// Marker agar sistem lain bisa query player entity.
#[derive(Component)]
pub struct PlayerMarker;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawn player entity di tengah map (dekat Void Core).
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        // Visual — MVP: cyan triangle (disimulasi sebagai rotated rectangle)
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_PLAYER,
                custom_size: Some(Vec2::new(14.0, 20.0)), // lebar x tinggi
                ..default()
            },
            transform: Transform::from_xyz(0.0, 60.0, 2.0), // sedikit di atas Void Core
            ..default()
        },
        // Physics
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::ball(PLAYER_COLLIDER_RADIUS),
        LockedAxes::ROTATION_LOCKED, // player tidak boleh rotate secara physics
        Damping {
            linear_damping: PLAYER_DAMPING,
            angular_damping: 0.0,
        },
        // Game components
        PlayerMarker,
        Player::default(),
        Health::new(100.0),
        Position(Vec2::new(0.0, 60.0)),
    ));
}

// ---------------------------------------------------------------------------
// Movement System (S0-04)
// ---------------------------------------------------------------------------

/// Baca input WASD, set velocity pada rapier RigidBody.
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut bevy_rapier2d::prelude::Velocity, &Player), With<PlayerMarker>>,
    time: Res<Time>,
) {
    let Ok((mut vel, player)) = player_q.get_single_mut() else { return };

    // Player tidak bisa bergerak saat dashing (handled di ability system S1-02)
    if player.is_dashing {
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }

    if dir != Vec2::ZERO {
        dir = dir.normalize();
    }

    vel.linvel = dir * PLAYER_SPEED;
    let _ = time; // dipakai saat kita tambah acceleration/deceleration nanti
}

// ---------------------------------------------------------------------------
// Mouse Facing System (S0-04)
// ---------------------------------------------------------------------------

/// Rotasi sprite player agar menghadap posisi mouse cursor.
fn player_mouse_facing(
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut player_q: Query<(&mut Transform, &mut Player), With<PlayerMarker>>,
) {
    let Ok(window) = window_q.get_single() else { return };
    let Ok((camera, cam_gtf)) = camera_q.get_single() else { return };
    let Ok((mut player_tf, mut player)) = player_q.get_single_mut() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(cam_gtf, cursor_pos) else { return };

    let player_pos = player_tf.translation.truncate();
    let dir = world_pos - player_pos;

    if dir.length_squared() > 1.0 {
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
        player_tf.rotation = Quat::from_rotation_z(angle);
        player.facing = angle;
    }
}

// ---------------------------------------------------------------------------
// Cooldown Tick (S0-04 — utility untuk semua ability)
// ---------------------------------------------------------------------------

/// Decrement semua ability cooldown setiap frame.
fn cooldown_tick(
    mut player_q: Query<&mut Player, With<PlayerMarker>>,
    time: Res<Time>,
) {
    let Ok(mut player) = player_q.get_single_mut() else { return };
    let dt = time.delta_seconds();
    let cd = &mut player.cooldowns;

    cd.melee = (cd.melee - dt).max(0.0);
    cd.dash = (cd.dash - dt).max(0.0);
    cd.grenade = (cd.grenade - dt).max(0.0);
    cd.void_explosion = (cd.void_explosion - dt).max(0.0);
    cd.repair_pulse = (cd.repair_pulse - dt).max(0.0);
}
