// Void Architect — plugins/player.rs
// Player controller: LMB click-to-move, mouse facing. [S0-04]
//
// Physics: KinematicVelocityBased — kita set velocity langsung,
// rapier gerakkan entity, tapi rotation TIDAK di-override rapier
// (kinematic body tidak apply torque/gravity). Ini yang benar untuk
// top-down game di mana rotation dikontrol manual via mouse facing.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::components::{Health, Player, Position};
use crate::GameState;

const PLAYER_SPEED: f32 = 180.0;
const PLAYER_COLLIDER_RADIUS: f32 = 10.0;
const ARRIVAL_THRESHOLD: f32 = 6.0;
const COLOR_PLAYER: Color = Color::srgb(0.0, 0.85, 0.85);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .insert_resource(MoveTarget::default())
            .add_systems(OnEnter(GameState::InRun), spawn_player)
            .add_systems(Update, (
                click_to_move_input,
                player_movement,
                player_mouse_facing,
                cooldown_tick,
            ).run_if(in_state(GameState::InRun)));
    }
}

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Resource, Default)]
pub struct MoveTarget(pub Option<Vec2>);

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: COLOR_PLAYER,
                custom_size: Some(Vec2::new(14.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 60.0, 2.0),
            ..default()
        },
        // KinematicVelocityBased: kita kontrol velocity secara langsung.
        // Rapier gerakkan posisi tapi TIDAK override rotation — aman untuk
        // manual mouse facing.
        RigidBody::KinematicVelocityBased,
        bevy_rapier2d::prelude::Velocity::default(),
        Collider::ball(PLAYER_COLLIDER_RADIUS),
        PlayerMarker,
        Player::default(),
        Health::new(100.0),
        Position(Vec2::new(0.0, 60.0)),
    ));
}

// ---------------------------------------------------------------------------
// LMB Click → set move target
// ---------------------------------------------------------------------------

fn click_to_move_input(
    mouse: Res<ButtonInput<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut move_target: ResMut<MoveTarget>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    let Ok(window) = window_q.get_single() else { return };
    let Ok((camera, cam_gtf)) = camera_q.get_single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(cam_gtf, cursor) else { return };

    move_target.0 = Some(world_pos);
}

// ---------------------------------------------------------------------------
// Move toward target
// ---------------------------------------------------------------------------

fn player_movement(
    mut player_q: Query<
        (&Transform, &mut bevy_rapier2d::prelude::Velocity, &Player),
        With<PlayerMarker>,
    >,
    mut move_target: ResMut<MoveTarget>,
) {
    let Ok((tf, mut vel, player)) = player_q.get_single_mut() else { return };

    if player.is_dashing { return; }

    let Some(target) = move_target.0 else {
        vel.linvel = Vec2::ZERO;
        return;
    };

    let current = tf.translation.truncate();
    let to_target = target - current;

    if to_target.length() < ARRIVAL_THRESHOLD {
        vel.linvel = Vec2::ZERO;
        move_target.0 = None;
    } else {
        vel.linvel = to_target.normalize() * PLAYER_SPEED;
    }
}

// ---------------------------------------------------------------------------
// Mouse facing — aman karena KinematicVelocityBased tidak override rotation
// ---------------------------------------------------------------------------

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

    let dir = world_pos - player_tf.translation.truncate();
    if dir.length_squared() > 1.0 {
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
        player_tf.rotation = Quat::from_rotation_z(angle);
        player.facing = angle;
    }
}

// ---------------------------------------------------------------------------
// Cooldown tick
// ---------------------------------------------------------------------------

fn cooldown_tick(
    mut player_q: Query<&mut Player, With<PlayerMarker>>,
    time: Res<Time>,
) {
    let Ok(mut player) = player_q.get_single_mut() else { return };
    let dt = time.delta_seconds();
    let cd = &mut player.cooldowns;
    cd.melee          = (cd.melee - dt).max(0.0);
    cd.dash           = (cd.dash - dt).max(0.0);
    cd.grenade        = (cd.grenade - dt).max(0.0);
    cd.void_explosion = (cd.void_explosion - dt).max(0.0);
    cd.repair_pulse   = (cd.repair_pulse - dt).max(0.0);
}
