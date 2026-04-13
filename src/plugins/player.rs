// Void Architect — plugins/player.rs
// Player controller + semua combat ability. [S0-04, S1-01, S1-02, S1-03, S1-04]
//
// ADR-05: KinematicVelocityBased — kita kontrol velocity & rotation manual,
//         rapier tidak override rotation (tidak ada torque/gravity).
// ADR-06: LMB click-to-move; LMB ke arah enemy = attack.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::components::{
    DamageEvent, Enemy, Health, Player, Position, Structure,
};
use crate::GameState;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PLAYER_SPEED: f32 = 180.0;
const PLAYER_COLLIDER_RADIUS: f32 = 10.0;
const ARRIVAL_THRESHOLD: f32 = 6.0;

// S1-01: Melee
const MELEE_RANGE: f32 = 45.0;
const MELEE_CONE_COS: f32 = 0.924; // cos(22.5°) — setengah dari cone 45°
const MELEE_DAMAGE: f32 = 25.0;
const MELEE_COOLDOWN: f32 = 0.4;
const MELEE_FLASH_DURATION: f32 = 0.08;

// S1-02: Dodge Dash
const DASH_DISTANCE: f32 = 90.0;
const DASH_IFRAME_DURATION: f32 = 0.27; // ~8 frame di 30fps
const DASH_COOLDOWN: f32 = 3.0;

// S1-03: Void Grenade
const GRENADE_SPEED: f32 = 320.0;
const GRENADE_AOE_RADIUS: f32 = 80.0;
const GRENADE_DAMAGE: f32 = 45.0;
const GRENADE_COOLDOWN: f32 = 12.0;
const GRENADE_LIFETIME: f32 = 2.0;

// S1-04: Repair Pulse
const REPAIR_PULSE_RANGE: f32 = 100.0;
const REPAIR_PULSE_HEAL: f32 = 20.0;
const REPAIR_PULSE_COOLDOWN: f32 = 20.0;

// Warna
const COLOR_PLAYER: Color = Color::srgb(0.0, 0.85, 0.85);
const COLOR_FLASH_WHITE: Color = Color::srgb(1.0, 1.0, 1.0);
const COLOR_GRENADE: Color = Color::srgb(0.5, 0.0, 1.0);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .insert_resource(MoveTarget::default())
            .insert_resource(DashTarget::default())
            .insert_resource(PendingRepairPulse(false))
            .add_event::<MeleeHitEvent>()
            .add_event::<GrenadeExplodeEvent>()
            .add_systems(OnEnter(GameState::InRun), spawn_player)
            .add_systems(Update, (
                handle_player_input,
                apply_dash_teleport,
                player_movement,
                player_mouse_facing,
                cooldown_tick,
                dash_iframe_tick,
                grenade_flight,
                grenade_explode,
                repair_pulse_effect,
                melee_flash_reset,
                process_melee_hits,
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Marker & Components
// ---------------------------------------------------------------------------

/// Marker player entity
#[derive(Component)]
pub struct PlayerMarker;

/// Komponen: player sedang dalam invincibility frame (setelah dash)
#[derive(Component)]
pub struct Invincible {
    pub timer: f32,
}

/// Grenade yang sedang terbang di udara
#[derive(Component)]
struct GrenadeProjectile {
    direction: Vec2,
    damage: f32,
    aoe_radius: f32,
    lifetime: f32,
}

/// Timer untuk flash putih setelah melee hit
#[derive(Component)]
struct MeleeFlashTimer(f32);

/// Resource: posisi tujuan dash (diproses di system terpisah untuk hindari borrow conflict)
#[derive(Resource, Default)]
struct DashTarget(Option<Vec2>);

/// Resource: repair pulse sedang menunggu diproses
#[derive(Resource)]
struct PendingRepairPulse(bool);

/// Resource: move target untuk click-to-move
#[derive(Resource, Default)]
pub struct MoveTarget(pub Option<Vec2>);

/// Event: melee berhasil hit enemy
#[derive(Event)]
pub struct MeleeHitEvent {
    pub target: Entity,
    pub damage: f32,
}

/// Event: grenade meledak di posisi ini
#[derive(Event)]
pub struct GrenadeExplodeEvent {
    pub position: Vec2,
    pub radius: f32,
    pub damage: f32,
}

// ---------------------------------------------------------------------------
// Spawn Player
// ---------------------------------------------------------------------------

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
        // ADR-05: Kinematic agar rotation tidak di-override rapier
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
// S1-01 ~ S1-04: Input Handler Terpusat
// ---------------------------------------------------------------------------

fn handle_player_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut player_q: Query<
        (Entity, &Transform, &mut Player),
        With<PlayerMarker>,
    >,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    mut move_target: ResMut<MoveTarget>,
    mut dash_target: ResMut<DashTarget>,
    mut melee_events: EventWriter<MeleeHitEvent>,
    mut pending_pulse: ResMut<PendingRepairPulse>,
    mut commands: Commands,
) {
    let Ok((player_entity, ptf, mut player)) = player_q.get_single_mut()
        else { return };

    let player_pos = ptf.translation.truncate();

    // Resolve posisi cursor di world space
    let cursor_world: Option<Vec2> = (|| {
        let window = window_q.get_single().ok()?;
        let (camera, cam_gtf) = camera_q.get_single().ok()?;
        let cursor = window.cursor_position()?;
        camera.viewport_to_world_2d(cam_gtf, cursor)
    })();

    // ========================
    // S1-01: MELEE ATTACK
    // Trigger: [F] atau LMB ke arah enemy dalam range
    // ========================
    let lmb_pressed = mouse.just_pressed(MouseButton::Left);

    // Cek apakah ada enemy dekat cursor (untuk LMB attack mode)
    let enemy_near_cursor = cursor_world.map_or(false, |cw| {
        enemy_q.iter().any(|(_, etf)| cw.distance(etf.translation.truncate()) < 30.0)
    });

    let melee_triggered = keyboard.just_pressed(KeyCode::KeyF)
        || (lmb_pressed && enemy_near_cursor);

    if melee_triggered && player.cooldowns.melee <= 0.0 {
        let angle = player.facing;
        // Facing vector: sprite menghadap atas → facing = angle dari y-axis
        // sin(angle) = komponen x, cos(angle) = komponen y
        let facing_vec = Vec2::new(angle.sin(), angle.cos());

        let mut hit_any = false;
        for (enemy_ent, etf) in &enemy_q {
            let to_enemy = etf.translation.truncate() - player_pos;
            let dist = to_enemy.length();
            if dist > MELEE_RANGE { continue; }

            // Cek apakah dalam cone 45°: dot > cos(22.5°)
            let in_cone = if dist > 0.01 {
                facing_vec.dot(to_enemy.normalize()) > MELEE_CONE_COS
            } else {
                true // kalau di posisi sama, selalu kena
            };

            if in_cone {
                melee_events.send(MeleeHitEvent {
                    target: enemy_ent,
                    damage: MELEE_DAMAGE,
                });
                hit_any = true;
            }
        }

        player.cooldowns.melee = MELEE_COOLDOWN;

        if hit_any {
            // Flash putih singkat sebagai hit feedback
            commands.entity(player_entity).insert(MeleeFlashTimer(MELEE_FLASH_DURATION));
        }
    }

    // LMB bukan attack → set move target (clamp ke batas map)
    if lmb_pressed && !melee_triggered {
        if let Some(world_pos) = cursor_world {
            let hw = crate::plugins::world::MAP_HALF_W - 20.0;
            let hh = crate::plugins::world::MAP_HALF_H - 20.0;
            let clamped = Vec2::new(world_pos.x.clamp(-hw, hw), world_pos.y.clamp(-hh, hh));
            move_target.0 = Some(clamped);
        }
    }

    // ========================
    // S1-02: DODGE DASH — [Space]
    // Teleport ke arah facing, 8-frame iframes
    // ========================
    if keyboard.just_pressed(KeyCode::Space) && player.cooldowns.dash <= 0.0 {
        let angle = player.facing;
        let dir = Vec2::new(angle.sin(), angle.cos()).normalize_or_zero();

        let target_pos = player_pos + dir * DASH_DISTANCE;
        // Clamp ke batas peta agar tidak keluar
        let hw = (crate::plugins::world::MAP_WIDTH as f32 * crate::plugins::world::TILE_SIZE) / 2.0 - 24.0;
        let hh = (crate::plugins::world::MAP_HEIGHT as f32 * crate::plugins::world::TILE_SIZE) / 2.0 - 24.0;
        let clamped = Vec2::new(target_pos.x.clamp(-hw, hw), target_pos.y.clamp(-hh, hh));

        dash_target.0 = Some(clamped);
        player.is_dashing = true;
        player.cooldowns.dash = DASH_COOLDOWN;
        move_target.0 = None; // batalkan move saat dash

        // Aktifkan invincibility frame
        commands.entity(player_entity).insert(Invincible {
            timer: DASH_IFRAME_DURATION,
        });
    }

    // ========================
    // S1-03: VOID GRENADE — [E]
    // Lempar ke arah cursor, AOE explosion
    // ========================
    if keyboard.just_pressed(KeyCode::KeyE) && player.cooldowns.grenade <= 0.0 {
        if let Some(target_pos) = cursor_world {
            let dir = (target_pos - player_pos).normalize_or_zero();
            player.cooldowns.grenade = GRENADE_COOLDOWN;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: COLOR_GRENADE,
                        custom_size: Some(Vec2::splat(8.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(player_pos.x, player_pos.y, 1.5),
                    ..default()
                },
                GrenadeProjectile {
                    direction: dir,
                    damage: GRENADE_DAMAGE,
                    aoe_radius: GRENADE_AOE_RADIUS,
                    lifetime: GRENADE_LIFETIME,
                },
            ));
        }
    }

    // ========================
    // S1-04: REPAIR PULSE — [R]
    // Heal struktur sekitar 20HP, AOE 100px
    // ========================
    if keyboard.just_pressed(KeyCode::KeyR) && player.cooldowns.repair_pulse <= 0.0 {
        player.cooldowns.repair_pulse = REPAIR_PULSE_COOLDOWN;
        pending_pulse.0 = true;
    }
}

// ---------------------------------------------------------------------------
// Apply Dash Teleport (system terpisah untuk hindari borrow conflict Transform)
// ---------------------------------------------------------------------------

fn apply_dash_teleport(
    mut player_q: Query<&mut Transform, With<PlayerMarker>>,
    mut dash_target: ResMut<DashTarget>,
    mut vel_q: Query<&mut bevy_rapier2d::prelude::Velocity, With<PlayerMarker>>,
) {
    let Some(target) = dash_target.0.take() else { return };

    let Ok(mut tf) = player_q.get_single_mut() else { return };
    let Ok(mut vel) = vel_q.get_single_mut() else { return };

    // Teleport instant ke posisi tujuan
    tf.translation.x = target.x;
    tf.translation.y = target.y;
    vel.linvel = Vec2::ZERO;
}

// ---------------------------------------------------------------------------
// Player Movement (click-to-move)
// ---------------------------------------------------------------------------

fn player_movement(
    mut player_q: Query<
        (&Transform, &mut bevy_rapier2d::prelude::Velocity, &Player),
        With<PlayerMarker>,
    >,
    mut move_target: ResMut<MoveTarget>,
) {
    let Ok((tf, mut vel, player)) = player_q.get_single_mut() else { return };

    // Jangan gerak saat dashing
    if player.is_dashing {
        vel.linvel = Vec2::ZERO;
        return;
    }

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
// Mouse Facing
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
        // ADR: angle dari y-axis karena sprite "menghadap atas"
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
        player_tf.rotation = Quat::from_rotation_z(angle);
        player.facing = angle;
    }
}

// ---------------------------------------------------------------------------
// Cooldown Tick
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

// ---------------------------------------------------------------------------
// S1-02: Dash Iframe Tick
// ---------------------------------------------------------------------------

fn dash_iframe_tick(
    mut commands: Commands,
    mut iframe_q: Query<(Entity, &mut Invincible, &mut Player)>,
    time: Res<Time>,
) {
    for (entity, mut iframe, mut player) in &mut iframe_q {
        iframe.timer -= time.delta_seconds();
        if iframe.timer <= 0.0 {
            commands.entity(entity).remove::<Invincible>();
            player.is_dashing = false;
        }
    }
}

// ---------------------------------------------------------------------------
// S1-03: Grenade Flight
// ---------------------------------------------------------------------------

fn grenade_flight(
    mut commands: Commands,
    mut grenade_q: Query<(Entity, &mut Transform, &mut GrenadeProjectile)>,
    mut explode_events: EventWriter<GrenadeExplodeEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (entity, mut tf, mut grenade) in &mut grenade_q {
        // Terbang lurus ke arah tujuan
        tf.translation.x += grenade.direction.x * GRENADE_SPEED * dt;
        tf.translation.y += grenade.direction.y * GRENADE_SPEED * dt;

        grenade.lifetime -= dt;
        if grenade.lifetime <= 0.0 {
            // Auto-explode saat lifetime habis
            explode_events.send(GrenadeExplodeEvent {
                position: tf.translation.truncate(),
                radius: grenade.aoe_radius,
                damage: grenade.damage,
            });
            commands.entity(entity).despawn();
        }
    }
}

fn grenade_explode(
    mut explode_events: EventReader<GrenadeExplodeEvent>,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for ev in explode_events.read() {
        for (enemy_ent, etf) in &enemy_q {
            let dist = ev.position.distance(etf.translation.truncate());
            if dist <= ev.radius {
                // Damage penuh tanpa falloff (GDD: AOE explosion)
                damage_events.send(DamageEvent {
                    target: enemy_ent,
                    amount: ev.damage,
                    from_turret: false,
                    from_npc: false,
                });
            }
        }
        // TODO S4-05: spawn particle explosion VFX
    }
}

// ---------------------------------------------------------------------------
// S1-04: Repair Pulse
// ---------------------------------------------------------------------------

fn repair_pulse_effect(
    mut pending: ResMut<PendingRepairPulse>,
    player_q: Query<&Transform, With<PlayerMarker>>,
    mut structure_q: Query<(&Transform, &mut Health), With<Structure>>,
) {
    if !pending.0 { return; }
    pending.0 = false;

    let Ok(ptf) = player_q.get_single() else { return };
    let player_pos = ptf.translation.truncate();

    let mut healed = 0u32;
    for (stf, mut health) in &mut structure_q {
        if player_pos.distance(stf.translation.truncate()) <= REPAIR_PULSE_RANGE {
            health.heal(REPAIR_PULSE_HEAL);
            healed += 1;
        }
    }

    if healed > 0 {
        bevy::log::info!("[RepairPulse] Healed {healed} struktur +{REPAIR_PULSE_HEAL}HP");
    }
    // TODO S4-05: spawn heal ring particle
}

// ---------------------------------------------------------------------------
// S1-01: Process Melee Hits (DamageEvent)
// ---------------------------------------------------------------------------

fn process_melee_hits(
    mut melee_events: EventReader<MeleeHitEvent>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for ev in melee_events.read() {
        damage_events.send(DamageEvent {
            target: ev.target,
            amount: ev.damage,
            from_turret: false,
            from_npc: false,
        });
    }
}

// ---------------------------------------------------------------------------
// Melee Flash Reset
// ---------------------------------------------------------------------------

fn melee_flash_reset(
    mut commands: Commands,
    mut flash_q: Query<(Entity, &mut Sprite, &mut MeleeFlashTimer)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut flash) in &mut flash_q {
        flash.0 -= time.delta_seconds();
        if flash.0 <= 0.0 {
            sprite.color = COLOR_PLAYER;
            commands.entity(entity).remove::<MeleeFlashTimer>();
        } else {
            sprite.color = COLOR_FLASH_WHITE;
        }
    }
}

