// Void Architect — plugins/combat.rs
// Hit detection, damage resolution, death events, loot drop, EXP grant.
// [S1-01 ~ S1-04 support, S1-13 death system]

use bevy::prelude::*;
use crate::GameState;
use crate::components::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DamageEvent>()
            .add_event::<EnemyDied>()
            .add_event::<VoidCoreDamaged>()
            .add_systems(Update, (
                process_damage_events,
                check_enemy_death,       // S1-13: enemy mati → event + despawn
                check_void_core_damage,  // monitor void core HP
                check_player_damage,     // player terkena damage (dari enemy AI nanti)
            ).run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Process Damage Events
// ---------------------------------------------------------------------------

fn process_damage_events(
    mut damage_events: EventReader<DamageEvent>,
    mut health_q: Query<&mut Health>,
    mut strategy: ResMut<StrategyTracker>,
    wall_q: Query<(), With<WallMarker>>,
) {
    for ev in damage_events.read() {
        let Ok(mut health) = health_q.get_mut(ev.target) else { continue };

        // Catat wall damage untuk adaptation tracker
        if wall_q.get(ev.target).is_ok() {
            strategy.wall_damage_total += ev.amount;
            strategy.structure_damage_total += ev.amount;
        }

        health.damage(ev.amount);
    }
}

// ---------------------------------------------------------------------------
// S1-13: Check Enemy Death
// ---------------------------------------------------------------------------

fn check_enemy_death(
    mut commands: Commands,
    enemy_q: Query<(Entity, &Transform, &Health, &Enemy)>,
    mut died_events: EventWriter<EnemyDied>,
    mut strategy: ResMut<StrategyTracker>,
    mut run_stats: ResMut<RunStats>,
) {
    for (entity, tf, health, enemy) in &enemy_q {
        if !health.is_dead() { continue; }

        let pos = tf.translation.truncate();

        died_events.send(EnemyDied {
            entity,
            position: pos,
            exp_reward: enemy.exp_reward,
            from_turret: false, // akan di-track lebih akurat nanti
            from_npc: false,
        });

        // Update stats
        strategy.total_kills += 1;
        run_stats.total_kills += 1;

        // Spawn loot — stone/scrap kecil
        spawn_loot(&mut commands, pos, entity);

        // Despawn enemy
        commands.entity(entity).despawn_recursive();

        bevy::log::debug!("[Combat] Enemy {:?} mati di {:?}", enemy.variant, pos);
    }
}

/// Spawn loot resource node kecil saat enemy mati
fn spawn_loot(commands: &mut Commands, pos: Vec2, _enemy_entity: Entity) {
    // Drop scrap kecil sebagai reward
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.70, 0.50, 0.20),
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 0.5),
            ..default()
        },
        ResourceNode {
            stone: 0,
            scrap: 1, // drop 1 scrap per kill
            void_crystal: 0,
            food: 0,
        },
    ));
}

// ---------------------------------------------------------------------------
// Check Void Core Damage
// ---------------------------------------------------------------------------

fn check_void_core_damage(
    core_q: Query<&Health, With<VoidCore>>,
    mut core_events: EventWriter<VoidCoreDamaged>,
    mut last_fraction: Local<f32>,
) {
    let Ok(health) = core_q.get_single() else { return };
    let frac = health.fraction();

    // Kirim event kalau HP berubah signifikan
    if (frac - *last_fraction).abs() > 0.01 {
        *last_fraction = frac;
        core_events.send(VoidCoreDamaged { remaining_fraction: frac });
    }
}

// ---------------------------------------------------------------------------
// Check Player Damage (dari enemy hits — enemy AI S1-09 akan pakai DamageEvent)
// ---------------------------------------------------------------------------

fn check_player_damage(
    player_q: Query<&Health, With<crate::plugins::player::PlayerMarker>>,
    invincible_q: Query<(), With<crate::plugins::player::Invincible>>,
) {
    let Ok(health) = player_q.get_single() else { return };

    // Kalau player dead → game over (handled di S3-07)
    if health.is_dead() {
        bevy::log::info!("[Combat] Player mati! HP = 0");
        // TODO S3-07: trigger game over state
    }

    // Invincible check: DamageEvent ke player perlu cek Invincible component
    // System lain (enemy attack) harus cek ini sebelum kirim DamageEvent ke player
    let _ = invincible_q;
}
