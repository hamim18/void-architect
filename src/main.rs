// Void Architect — main.rs
// Entry point: App builder, game state machine, plugin registration.
// S3 additions: RunEndEvent, LevelUpState, BossState, SanctumState

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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InRun,
    MetaScreen,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Void Architect".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
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
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

#[derive(Component)]
pub struct MainCamera;
