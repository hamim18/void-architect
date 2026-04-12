// Void Architect — main.rs
// Entry point: App builder, game state machine, plugin registration.

use bevy::prelude::*;

pub mod components;

mod plugins {
    pub mod audio;
    pub mod colony;
    pub mod combat;
    pub mod enemies;
    pub mod phase;
    pub mod player;
    pub mod progression;
    pub mod structures;
    pub mod ui;
    pub mod world;
}

use plugins::{
    audio::AudioPlugin,
    colony::ColonyPlugin,
    combat::CombatPlugin,
    enemies::EnemiesPlugin,
    phase::PhasePlugin,
    player::PlayerPlugin,
    progression::ProgressionPlugin,
    structures::StructuresPlugin,
    ui::UiPlugin,
    world::WorldPlugin,
};

// ---------------------------------------------------------------------------
// Game State Machine
// ---------------------------------------------------------------------------

/// Top-level game states. Bevy uses this to gate which systems run.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// Title screen and main menu.
    #[default]
    MainMenu,
    /// Active run — all gameplay systems active.
    InRun,
    /// Between-run meta screen (Architect's Sanctum).
    MetaScreen,
}

// ---------------------------------------------------------------------------
// App Entry Point
// ---------------------------------------------------------------------------

fn main() {
    App::new()
        // --- Bevy built-ins ---
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Void Architect".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // --- State machine ---
        .init_state::<GameState>()
        // --- Game plugins (each owns its systems + resources) ---
        .add_plugins((
            WorldPlugin,
            PlayerPlugin,
            CombatPlugin,
            StructuresPlugin,
            EnemiesPlugin,
            ColonyPlugin,
            ProgressionPlugin,
            PhasePlugin,
            AudioPlugin,
            UiPlugin,
        ))
        // --- Startup ---
        .add_systems(Startup, setup_camera)
        .run();
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// Spawn the main 2D camera. WorldPlugin will attach follow logic to it.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));
}

/// Marker component so systems can query the unique main camera.
#[derive(Component)]
pub struct MainCamera;
