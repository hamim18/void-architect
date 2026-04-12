// Void Architect — plugins/ui.rs
// HUD + Debug UI. Full HUD di Sprint 4 (S4-01).
// Sprint 1 addition: build mode indicator, cooldown display.

use bevy::prelude::*;
use crate::components::*;
use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnEnter(GameState::InRun), spawn_debug_hud)
            .add_systems(Update,
                main_menu_input.run_if(in_state(GameState::MainMenu)))
            .add_systems(Update,
                update_debug_hud.run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct HudPhase;
#[derive(Component)] struct HudResources;
#[derive(Component)] struct HudPlayerHp;
#[derive(Component)] struct HudCooldowns;
#[derive(Component)] struct HudBuildMode;

// ---------------------------------------------------------------------------
// Main Menu
// ---------------------------------------------------------------------------

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgb(0.05, 0.04, 0.08).into(),
            ..default()
        },
        MainMenuRoot,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "VOID ARCHITECT",
            TextStyle { font_size: 52.0, color: Color::srgb(0.0, 0.85, 0.85), ..default() },
        ));
        parent.spawn(TextBundle::from_section(
            "\n[ENTER] Start Run",
            TextStyle { font_size: 22.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() },
        ));
        parent.spawn(TextBundle::from_section(
            "[ESC] Quit",
            TextStyle { font_size: 18.0, color: Color::srgb(0.4, 0.4, 0.4), ..default() },
        ));
        parent.spawn(TextBundle::from_section(
            "\nKontrol:\n[WASD/LMB] Gerak | [F] Melee | [Space] Dash | [E] Grenade | [R] Repair\n[B] Build Mode | [W/T/G/H] Pilih Struktur saat Build",
            TextStyle { font_size: 14.0, color: Color::srgb(0.35, 0.35, 0.45), ..default() },
        ));
    });
}

fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
    menu_q: Query<Entity, With<MainMenuRoot>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        for entity in &menu_q {
            commands.entity(entity).despawn_recursive();
        }
        next_state.set(GameState::InRun);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit.send(AppExit::Success);
    }
}

// ---------------------------------------------------------------------------
// Debug HUD — Sprint 1 extended
// ---------------------------------------------------------------------------

fn spawn_debug_hud(mut commands: Commands) {
    // Phase & Day info (top center-ish)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Day 1 | DAY | 3:00",
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudPhase,
    ));

    // Resources (top right)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Stone: 50 | Scrap: 20 | Crystal: 0 | Food: 10",
                TextStyle { font_size: 15.0, color: Color::srgb(0.8, 0.8, 0.5), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudResources,
    ));

    // Player HP (top left, bawah phase)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "HP: 100/100",
                TextStyle { font_size: 15.0, color: Color::srgb(0.3, 0.9, 0.3), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(32.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudPlayerHp,
    ));

    // Cooldown display (bottom left)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "[F]Melee:✓ [Space]Dash:✓ [E]Grenade:✓ [R]Pulse:✓",
                TextStyle { font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.9), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(32.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudCooldowns,
    ));

    // Build mode indicator (bottom center)
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle { font_size: 15.0, color: Color::srgb(0.5, 1.0, 0.5), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudBuildMode,
    ));
}

fn update_debug_hud(
    phase_timer: Res<PhaseTimer>,
    resources: Res<PlayerResources>,
    build_mode: Res<crate::plugins::structures::BuildMode>,
    player_q: Query<(&Health, &Player), With<crate::plugins::player::PlayerMarker>>,
    mut phase_q: Query<&mut Text,
        (With<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut res_q: Query<&mut Text,
        (With<HudResources>, Without<HudPhase>, Without<HudPlayerHp>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut hp_q: Query<&mut Text,
        (With<HudPlayerHp>, Without<HudPhase>, Without<HudResources>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut cd_q: Query<&mut Text,
        (With<HudCooldowns>, Without<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudBuildMode>)>,
    mut bm_q: Query<&mut Text,
        (With<HudBuildMode>, Without<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudCooldowns>)>,
) {
    let phase_name = match phase_timer.phase {
        Phase::Day   => "DAY",
        Phase::Night => "NIGHT",
    };
    let mins = (phase_timer.remaining / 60.0) as u32;
    let secs = (phase_timer.remaining % 60.0) as u32;

    // Phase
    if let Ok(mut t) = phase_q.get_single_mut() {
        t.sections[0].value = format!(
            "Day {} | {} | {}:{:02} | Wave {}",
            phase_timer.day, phase_name, mins, secs, phase_timer.wave_num
        );
    }

    // Resources
    if let Ok(mut t) = res_q.get_single_mut() {
        t.sections[0].value = format!(
            "Stone:{} Scrap:{} Crystal:{} Food:{}",
            resources.stone, resources.scrap, resources.void_crystal, resources.food
        );
    }

    // Player HP + Level
    if let Ok((hp, player)) = player_q.get_single() {
        if let Ok(mut t) = hp_q.get_single_mut() {
            t.sections[0].value = format!(
                "HP: {:.0}/{:.0} | Lv.{} EXP:{}/{}",
                hp.current, hp.max, player.level, player.exp, player.exp_next
            );
        }

        // Cooldowns
        if let Ok(mut t) = cd_q.get_single_mut() {
            let cd = &player.cooldowns;
            let fmt_cd = |v: f32| if v <= 0.0 { "✓".to_string() } else { format!("{:.1}s", v) };
            t.sections[0].value = format!(
                "[F]Melee:{} [Spc]Dash:{} [E]Nade:{} [R]Pulse:{}",
                fmt_cd(cd.melee), fmt_cd(cd.dash), fmt_cd(cd.grenade), fmt_cd(cd.repair_pulse)
            );
        }
    }

    // Build mode
    if let Ok(mut t) = bm_q.get_single_mut() {
        if build_mode.active {
            let sel_name = build_mode.selected
                .map(|s| s.name())
                .unwrap_or("None");
            t.sections[0].value = format!(
                "[BUILD MODE] {} | W=Wall T=Turret G=Farm H=House | LMB=Place ESC=Cancel",
                sel_name
            );
        } else {
            t.sections[0].value = "[B] Build Mode".to_string();
        }
    }
}
