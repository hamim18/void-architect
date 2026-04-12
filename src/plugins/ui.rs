// Void Architect — plugins/ui.rs
// HUD, modals, menus. S0 scope: debug HUD minimal.
// Full HUD: Sprint 4 (S4-01)

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
                update_debug_hud_in_run.run_if(in_state(GameState::InRun)));
    }
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct DebugHudPhase;
#[derive(Component)] struct DebugHudResources;
#[derive(Component)] struct DebugHudPlayerHp;

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
// Debug HUD
// ---------------------------------------------------------------------------

fn spawn_debug_hud(mut commands: Commands) {
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
        DebugHudPhase,
    ));

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Stone: 50 | Scrap: 20 | Crystal: 0 | Food: 10",
                TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.8, 0.5), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        DebugHudResources,
    ));

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "HP: 100/100",
                TextStyle { font_size: 16.0, color: Color::srgb(0.3, 0.9, 0.3), ..default() },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(32.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        DebugHudPlayerHp,
    ));
}

fn update_debug_hud_in_run(
    phase_timer: Res<PhaseTimer>,
    resources: Res<PlayerResources>,
    player_q: Query<&Health, With<crate::plugins::player::PlayerMarker>>,
    mut phase_q: Query<&mut Text,
        (With<DebugHudPhase>, Without<DebugHudResources>, Without<DebugHudPlayerHp>)>,
    mut res_q: Query<&mut Text,
        (With<DebugHudResources>, Without<DebugHudPhase>, Without<DebugHudPlayerHp>)>,
    mut hp_q: Query<&mut Text,
        (With<DebugHudPlayerHp>, Without<DebugHudPhase>, Without<DebugHudResources>)>,
) {
    let phase_name = match phase_timer.phase {
        Phase::Day => "DAY",
        Phase::Night => "NIGHT",
    };
    let mins = (phase_timer.remaining / 60.0) as u32;
    let secs = (phase_timer.remaining % 60.0) as u32;

    if let Ok(mut t) = phase_q.get_single_mut() {
        t.sections[0].value = format!(
            "Day {} | {} | {}:{:02}",
            phase_timer.day, phase_name, mins, secs
        );
    }

    if let Ok(mut t) = res_q.get_single_mut() {
        t.sections[0].value = format!(
            "Stone: {} | Scrap: {} | Crystal: {} | Food: {}",
            resources.stone, resources.scrap, resources.void_crystal, resources.food
        );
    }

    if let Ok(hp) = player_q.get_single() {
        if let Ok(mut t) = hp_q.get_single_mut() {
            t.sections[0].value = format!("HP: {:.0}/{:.0}", hp.current, hp.max);
        }
    }
}
