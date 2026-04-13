// Void Architect — plugins/ui.rs
// HUD, Level-Up Modal (S3-02), MetaScreen/Sanctum UI (S3-10), Run-End Screen.
// [S2 colony HUD, S3-02, S3-06 void core display, S3-10]

use bevy::prelude::*;
use crate::components::*;
use crate::GameState;
use crate::plugins::progression::{
    LevelUpState, PerkSelected, RunEndState, SanctumPurchaseEvent,
    SanctumState, SanctumUnlock, ALL_SANCTUM_UNLOCKS,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu),  spawn_main_menu)
            .add_systems(OnEnter(GameState::InRun),     spawn_hud)
            .add_systems(OnExit(GameState::InRun),      despawn_hud)
            .add_systems(OnEnter(GameState::MetaScreen),spawn_meta_screen)
            .add_systems(OnExit(GameState::MetaScreen), despawn_meta_screen)
            .add_systems(Update, main_menu_input.run_if(in_state(GameState::MainMenu)))
            .add_systems(Update, (
                update_hud,
                update_colony_hud,
                update_rescue_prompt,
                update_void_core_hud,
                // S3-02: Level-up modal
                update_level_up_modal,
                level_up_input,
                // Run-end overlay
                update_run_end_overlay,
            ).run_if(in_state(GameState::InRun)))
            .add_systems(Update, (
                // S3-10: Sanctum input
                sanctum_input,
                update_sanctum_ui,
            ).run_if(in_state(GameState::MetaScreen)));
    }
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)] struct MainMenuRoot;
#[derive(Component)] struct HudRoot;
#[derive(Component)] struct HudPhase;
#[derive(Component)] struct HudResources;
#[derive(Component)] struct HudPlayerHp;
#[derive(Component)] struct HudCooldowns;
#[derive(Component)] struct HudBuildMode;
#[derive(Component)] struct HudColony;
#[derive(Component)] struct HudRescuePrompt;
#[derive(Component)] struct HudVoidCore;
#[derive(Component)] struct LevelUpModal;
#[derive(Component)] struct LevelUpPerkCard(usize);
#[derive(Component)] struct RunEndOverlay;
#[derive(Component)] struct MetaRoot;
#[derive(Component)] struct SanctumShardText;
#[derive(Component)] struct SanctumUnlockRow(usize);
#[derive(Component)] struct SanctumMessage;

// ---------------------------------------------------------------------------
// Main Menu
// ---------------------------------------------------------------------------

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgb(0.04, 0.03, 0.07).into(),
            ..default()
        },
        MainMenuRoot,
    )).with_children(|p| {
        p.spawn(TextBundle::from_section("VOID ARCHITECT",
            TextStyle { font_size: 56.0, color: Color::srgb(0.0, 0.9, 0.9), ..default() }));
        p.spawn(TextBundle::from_section("Survival Colony Defense · Roguelite\n",
            TextStyle { font_size: 18.0, color: Color::srgb(0.4, 0.4, 0.6), ..default() }));
        p.spawn(TextBundle::from_section("[ENTER] New Run",
            TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.8, 0.8), ..default() }));
        p.spawn(TextBundle::from_section("[ESC] Quit\n",
            TextStyle { font_size: 18.0, color: Color::srgb(0.5, 0.5, 0.5), ..default() }));
        p.spawn(TextBundle::from_section(
            "Controls: [LMB] Move  [F] Melee  [Space] Dash\n[E] Grenade  [R] Repair  [B] Build Mode",
            TextStyle { font_size: 13.0, color: Color::srgb(0.3, 0.3, 0.4), ..default() }));
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
        for e in &menu_q { commands.entity(e).despawn_recursive(); }
        next_state.set(GameState::InRun);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit.send(AppExit::Success);
    }
}

// ---------------------------------------------------------------------------
// In-Run HUD
// ---------------------------------------------------------------------------

fn spawn_hud(mut commands: Commands) {
    commands.spawn((NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ..default()
    }, HudRoot)).with_children(|root| {
        // Phase + day (top-left)
        root.spawn((TextBundle {
            text: Text::from_section("Day 1 | DAY | 3:00", TextStyle { font_size: 17.0, color: Color::WHITE, ..default() }),
            style: Style { position_type: PositionType::Absolute, top: Val::Px(8.0), left: Val::Px(8.0), ..default() },
            ..default()
        }, HudPhase));
        // Resources (top-right)
        root.spawn((TextBundle {
            text: Text::from_section("Stone:50 Scrap:20 Crystal:0 Food:10", TextStyle { font_size: 14.0, color: Color::srgb(0.85, 0.85, 0.5), ..default() }),
            style: Style { position_type: PositionType::Absolute, top: Val::Px(8.0), right: Val::Px(8.0), ..default() },
            ..default()
        }, HudResources));
        // Player HP + Level (top-left row 2)
        root.spawn((TextBundle {
            text: Text::from_section("HP: 100/100 | Lv.1 | EXP: 0/500", TextStyle { font_size: 14.0, color: Color::srgb(0.3, 0.9, 0.3), ..default() }),
            style: Style { position_type: PositionType::Absolute, top: Val::Px(30.0), left: Val::Px(8.0), ..default() },
            ..default()
        }, HudPlayerHp));
        // Cooldowns (bottom-left)
        root.spawn((TextBundle {
            text: Text::from_section("[F]Melee:✓ [Spc]Dash:✓ [E]Nade:✓ [R]Pulse:✓", TextStyle { font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.9), ..default() }),
            style: Style { position_type: PositionType::Absolute, bottom: Val::Px(32.0), left: Val::Px(8.0), ..default() },
            ..default()
        }, HudCooldowns));
        // Build mode (bottom-left row 2)
        root.spawn((TextBundle {
            text: Text::from_section("[B] Build Mode", TextStyle { font_size: 14.0, color: Color::srgb(0.5, 1.0, 0.5), ..default() }),
            style: Style { position_type: PositionType::Absolute, bottom: Val::Px(8.0), left: Val::Px(8.0), ..default() },
            ..default()
        }, HudBuildMode));
        // Colony (bottom-right)
        root.spawn((TextBundle {
            text: Text::from_section("Pop:0/0 | Morale:70", TextStyle { font_size: 13.0, color: Color::srgb(0.6, 0.9, 0.6), ..default() }),
            style: Style { position_type: PositionType::Absolute, bottom: Val::Px(32.0), right: Val::Px(8.0), ..default() },
            ..default()
        }, HudColony));
        // Rescue prompt (center-bottom)
        root.spawn((TextBundle {
            text: Text::from_section("", TextStyle { font_size: 17.0, color: Color::srgb(1.0, 0.9, 0.2), ..default() }),
            style: Style { position_type: PositionType::Absolute, bottom: Val::Px(80.0), left: Val::Percent(25.0), ..default() },
            ..default()
        }, HudRescuePrompt));
        // Void Core HP (center-top)
        root.spawn((TextBundle {
            text: Text::from_section("[ VOID CORE: 500/500 ]", TextStyle { font_size: 15.0, color: Color::srgb(0.6, 0.0, 1.0), ..default() }),
            style: Style { position_type: PositionType::Absolute, top: Val::Px(8.0), left: Val::Percent(40.0), ..default() },
            ..default()
        }, HudVoidCore));

        // S3-02: Level-up modal (hidden by default)
        spawn_level_up_modal(root);

        // Run-end overlay (hidden)
        root.spawn((TextBundle {
            text: Text::from_section("", TextStyle { font_size: 36.0, color: Color::WHITE, ..default() }),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(35.0), left: Val::Percent(25.0), right: Val::Percent(25.0),
                ..default()
            },
            ..default()
        }, RunEndOverlay));
    });
}

fn despawn_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

// ---------------------------------------------------------------------------
// S3-02: Level-Up Modal
// ---------------------------------------------------------------------------

fn spawn_level_up_modal(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                display: Display::None, // hidden by default
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0), left: Val::Percent(15.0), right: Val::Percent(15.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            background_color: Color::srgba(0.05, 0.04, 0.12, 0.95).into(),
            ..default()
        },
        LevelUpModal,
    )).with_children(|modal| {
        modal.spawn(TextBundle::from_section(
            "LEVEL UP — Choose an Upgrade",
            TextStyle { font_size: 22.0, color: Color::srgb(0.0, 0.9, 0.9), ..default() },
        ));
        // 3 perk cards
        for i in 0..3 {
            modal.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0), padding: UiRect::all(Val::Px(10.0)),
                        flex_direction: FlexDirection::Column, ..default()
                    },
                    background_color: Color::srgb(0.10, 0.08, 0.18).into(),
                    ..default()
                },
                LevelUpPerkCard(i),
            )).with_children(|card| {
                card.spawn(TextBundle::from_section(
                    format!("[{}] — (loading...)", i + 1),
                    TextStyle { font_size: 16.0, color: Color::srgb(0.9, 0.85, 1.0), ..default() },
                ));
            });
        }
        modal.spawn(TextBundle::from_section(
            "Press [1] [2] [3] to select",
            TextStyle { font_size: 13.0, color: Color::srgb(0.5, 0.5, 0.6), ..default() },
        ));
    });
}

fn update_level_up_modal(
    level_up: Res<LevelUpState>,
    mut modal_q: Query<&mut Style, With<LevelUpModal>>,
    mut card_q: Query<(&LevelUpPerkCard, &Children)>,
    mut text_q: Query<&mut Text>,
) {
    let Ok(mut style) = modal_q.get_single_mut() else { return };

    if level_up.is_active {
        style.display = Display::Flex;
        // Update card text
        for (card, children) in &card_q {
            let idx = card.0;
            let perk = level_up.offered[idx];
            let label = if let Some(p) = perk {
                format!("[{}] {} ({})\n    {}", idx + 1, p.name(), p.tree(), p.description())
            } else {
                format!("[{}] — (empty)", idx + 1)
            };
            for &child in children.iter() {
                if let Ok(mut t) = text_q.get_mut(child) {
                    t.sections[0].value = label.clone();
                }
            }
        }
    } else {
        style.display = Display::None;
    }
}

fn level_up_input(
    level_up: Res<LevelUpState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut perk_events: EventWriter<PerkSelected>,
) {
    if !level_up.is_active { return; }

    let picked = if keyboard.just_pressed(KeyCode::Digit1) { Some(0) }
        else if keyboard.just_pressed(KeyCode::Digit2) { Some(1) }
        else if keyboard.just_pressed(KeyCode::Digit3) { Some(2) }
        else { None };

    if let Some(idx) = picked {
        if let Some(perk) = level_up.offered[idx] {
            perk_events.send(PerkSelected { perk });
        }
    }
}

// ---------------------------------------------------------------------------
// HUD update systems
// ---------------------------------------------------------------------------

fn update_hud(
    phase_timer: Res<PhaseTimer>,
    resources: Res<PlayerResources>,
    build_mode: Res<crate::plugins::structures::BuildMode>,
    player_q: Query<(&Health, &Player), With<crate::plugins::player::PlayerMarker>>,
    mut phase_q: Query<&mut Text, (With<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut res_q: Query<&mut Text, (With<HudResources>, Without<HudPhase>, Without<HudPlayerHp>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut hp_q: Query<&mut Text, (With<HudPlayerHp>, Without<HudPhase>, Without<HudResources>, Without<HudCooldowns>, Without<HudBuildMode>)>,
    mut cd_q: Query<&mut Text, (With<HudCooldowns>, Without<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudBuildMode>)>,
    mut bm_q: Query<&mut Text, (With<HudBuildMode>, Without<HudPhase>, Without<HudResources>, Without<HudPlayerHp>, Without<HudCooldowns>)>,
) {
    let phase_name = match phase_timer.phase { Phase::Day => "DAY", Phase::Night => "NIGHT" };
    let mins = (phase_timer.remaining / 60.0) as u32;
    let secs = (phase_timer.remaining % 60.0) as u32;

    if let Ok(mut t) = phase_q.get_single_mut() {
        t.sections[0].value = format!("Day {} | {} | {}:{:02} | Wave {}", phase_timer.day, phase_name, mins, secs, phase_timer.wave_num);
    }
    if let Ok(mut t) = res_q.get_single_mut() {
        t.sections[0].value = format!("Stone:{} Scrap:{} Crystal:{} Food:{}", resources.stone, resources.scrap, resources.void_crystal, resources.food);
    }
    if let Ok((hp, player)) = player_q.get_single() {
        if let Ok(mut t) = hp_q.get_single_mut() {
            t.sections[0].value = format!("HP:{:.0}/{:.0} | Lv.{} | EXP:{}/{}", hp.current, hp.max, player.level, player.exp, player.exp_next);
        }
        if let Ok(mut t) = cd_q.get_single_mut() {
            let cd = &player.cooldowns;
            let f = |v: f32| if v <= 0.0 { "✓".into() } else { format!("{:.1}s", v) };
            t.sections[0].value = format!("[F]:{} [Spc]:{} [E]:{} [R]:{}", f(cd.melee), f(cd.dash), f(cd.grenade), f(cd.repair_pulse));
        }
    }
    if let Ok(mut t) = bm_q.get_single_mut() {
        if build_mode.active {
            let sel = build_mode.selected.map(|s| s.name()).unwrap_or("None");
            t.sections[0].value = format!("[BUILD: {}] W=Wall T=Turret G=Farm H=House | ESC=cancel", sel);
        } else {
            t.sections[0].value = "[B] Build Mode".into();
        }
    }
}

fn update_colony_hud(
    colony: Res<ColonyState>,
    npc_q: Query<&Npc>,
    mut q: Query<&mut Text, With<HudColony>>,
) {
    let Ok(mut t) = q.get_single_mut() else { return };
    let farmers  = npc_q.iter().filter(|n| n.role == NpcRole::Farmer).count();
    let builders = npc_q.iter().filter(|n| n.role == NpcRole::Builder).count();
    let guards   = npc_q.iter().filter(|n| n.role == NpcRole::Guard).count();
    let morale_tag = if colony.morale < 30.0 { "⚠" } else if colony.morale > 80.0 { "★" } else { "" };
    t.sections[0].value = format!(
        "Pop:{}/{} F:{} B:{} G:{} Food:{} | {}Morale:{:.0}{}",
        colony.population, colony.max_population, farmers, builders, guards, colony.food,
        morale_tag, colony.morale,
        if colony.starvation_days > 0 { format!(" STARVE:{}d", colony.starvation_days) } else { String::new() },
    );
}

fn update_rescue_prompt(
    rescue: Res<crate::plugins::colony::RescueState>,
    mut q: Query<&mut Text, With<HudRescuePrompt>>,
) {
    let Ok(mut t) = q.get_single_mut() else { return };
    if rescue.prompt_active {
        let role = match rescue.pending_role { NpcRole::Farmer=>"Farmer", NpcRole::Builder=>"Builder", NpcRole::Guard=>"Guard", _=>"Survivor" };
        t.sections[0].value = format!("[ {} FOUND ] [R] Rescue  [N] Skip  ({:.0}s)", role, rescue.timeout_timer);
    } else {
        t.sections[0].value.clear();
    }
}

// S3-06: Void Core HP display
fn update_void_core_hud(
    core_q: Query<&Health, With<VoidCore>>,
    mut q: Query<&mut Text, With<HudVoidCore>>,
) {
    let Ok(mut t) = q.get_single_mut() else { return };
    if let Ok(hp) = core_q.get_single() {
        let pct = hp.fraction() * 100.0;
        let color_hint = if pct < 30.0 { "⚠ " } else { "" };
        t.sections[0].value = format!("{}[ VOID CORE: {:.0}/{:.0} ]", color_hint, hp.current, hp.max);
    }
}

// S3-07: run-end overlay
fn update_run_end_overlay(
    run_end: Res<RunEndState>,
    run_stats: Res<RunStats>,
    meta: Res<MetaProgress>,
    mut q: Query<&mut Text, With<RunEndOverlay>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut t) = q.get_single_mut() else { return };
    if run_end.finished {
        let result = if run_end.victory { "VICTORY" } else { "DEFEAT" };
        let shards = crate::plugins::progression::calculate_void_shards(&run_stats);
        t.sections[0].value = format!(
            "{}\nDays: {} | Kills: {} | Bosses: {}\n+{} Void Shards (Total: {})\n\n[ENTER] Continue",
            result, run_stats.days_survived, run_stats.total_kills,
            run_stats.bosses_defeated, shards, meta.void_shards,
        );
        if keyboard.just_pressed(KeyCode::Enter) {
            next_state.set(GameState::MetaScreen);
        }
    }
}

// ---------------------------------------------------------------------------
// S3-10: MetaScreen / Architect's Sanctum
// ---------------------------------------------------------------------------

fn spawn_meta_screen(
    mut commands: Commands,
    meta: Res<MetaProgress>,
    run_stats: Res<RunStats>,
    run_end: Res<RunEndState>,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            background_color: Color::srgb(0.04, 0.03, 0.08).into(),
            ..default()
        },
        MetaRoot,
    )).with_children(|root| {
        let title = if run_end.finished {
            if run_end.victory { "VICTORY — ARCHITECT'S SANCTUM" } else { "DEFEAT — ARCHITECT'S SANCTUM" }
        } else { "ARCHITECT'S SANCTUM" };

        root.spawn(TextBundle::from_section(title,
            TextStyle { font_size: 28.0, color: Color::srgb(0.0, 0.9, 0.9), ..default() }));

        // Shard balance
        root.spawn((
            TextBundle::from_section(
                format!("Void Shards: {}", meta.void_shards),
                TextStyle { font_size: 20.0, color: Color::srgb(0.8, 0.5, 1.0), ..default() },
            ),
            SanctumShardText,
        ));

        root.spawn(TextBundle::from_section(
            "── Unlocks (press key to purchase) ──",
            TextStyle { font_size: 14.0, color: Color::srgb(0.4, 0.4, 0.5), ..default() },
        ));

        // 5 unlock rows
        for (i, &unlock) in ALL_SANCTUM_UNLOCKS.iter().enumerate() {
            let owned = unlock.is_owned(&meta);
            let color = if owned { Color::srgb(0.3, 0.8, 0.3) } else { Color::srgb(0.75, 0.75, 0.75) };
            let status = if owned { "[OWNED]" } else { "" };
            root.spawn((
                TextBundle::from_section(
                    format!("[{}] {} {}", i + 1, unlock.label(), status),
                    TextStyle { font_size: 15.0, color, ..default() },
                ),
                SanctumUnlockRow(i),
            ));
        }

        // Message feedback
        root.spawn((
            TextBundle::from_section("", TextStyle { font_size: 14.0, color: Color::srgb(1.0, 0.8, 0.2), ..default() }),
            SanctumMessage,
        ));

        root.spawn(TextBundle::from_section(
            "\n[ENTER] New Run    [ESC] Quit",
            TextStyle { font_size: 17.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() },
        ));
    });
}

fn despawn_meta_screen(mut commands: Commands, q: Query<Entity, With<MetaRoot>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn sanctum_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut purchase_events: EventWriter<SanctumPurchaseEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    let keys = [
        (KeyCode::Digit1, SanctumUnlock::StartingTurret),
        (KeyCode::Digit2, SanctumUnlock::StartingWalls),
        (KeyCode::Digit3, SanctumUnlock::VoidAffinity),
        (KeyCode::Digit4, SanctumUnlock::ColonyBond),
        (KeyCode::Digit5, SanctumUnlock::UnlockSentinel),
    ];
    for (key, unlock) in &keys {
        if keyboard.just_pressed(*key) {
            purchase_events.send(SanctumPurchaseEvent { unlock: *unlock });
        }
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::InRun);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit.send(AppExit::Success);
    }
}

fn update_sanctum_ui(
    meta: Res<MetaProgress>,
    sanctum_state: Res<SanctumState>,
    mut shard_q: Query<&mut Text, (With<SanctumShardText>, Without<SanctumMessage>, Without<SanctumUnlockRow>)>,
    mut msg_q: Query<&mut Text, (With<SanctumMessage>, Without<SanctumShardText>, Without<SanctumUnlockRow>)>,
    mut row_q: Query<(&SanctumUnlockRow, &mut Text), Without<SanctumShardText>>,
) {
    if let Ok(mut t) = shard_q.get_single_mut() {
        t.sections[0].value = format!("Void Shards: {}", meta.void_shards);
    }
    if let Ok(mut t) = msg_q.get_single_mut() {
        t.sections[0].value = sanctum_state.last_message.clone();
    }
    for (row, mut t) in &mut row_q {
        let unlock = ALL_SANCTUM_UNLOCKS[row.0];
        let owned = unlock.is_owned(&meta);
        let status = if owned { "[OWNED]" } else { "" };
        t.sections[0].value = format!("[{}] {} {}", row.0 + 1, unlock.label(), status);
        t.sections[0].style.color = if owned { Color::srgb(0.3, 0.8, 0.3) } else { Color::srgb(0.75, 0.75, 0.75) };
    }
}
