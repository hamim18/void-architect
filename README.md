# Void Architect

> Survival Colony Defense Roguelite — Top-Down

[![CI](https://github.com/YOUR_USERNAME/void-architect/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/void-architect/actions)

## Status

🚧 **MVP Sprint — Week 1/8** | Sprint S0 In Progress

| Sprint | Focus | Status |
|--------|-------|--------|
| S0 — Foundation | Scaffold, ECS, tilemap, player, phase timer | 🔄 In Progress |
| S1 — Combat & Structures | Melee, abilities, enemies, wave spawner | ⏳ Pending |
| S2 — Colony Systems | NPC roles, hunger, morale | ⏳ Pending |
| S3 — Progression & Boss | Level-up, perks, Boss 1, meta-save | ⏳ Pending |
| S4 — Polish & Ship | HUD, audio, particles, playtest | ⏳ Pending |

## Tech Stack

- **Engine**: Bevy 0.14 (Rust ECS)
- **Physics**: bevy_rapier2d
- **Serialization**: serde_json (meta-save)
- **Proc Gen**: noise (Perlin) + rand

## Dev Setup

```bash
# Clone dan jalankan
git clone https://github.com/YOUR_USERNAME/void-architect
cd void-architect
cargo run

# Release build
cargo build --release
```

### Linux Dependencies

```bash
sudo apt-get install libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev pkg-config
```

## Controls (MVP)

| Key | Action |
|-----|--------|
| WASD / Arrow Keys | Move |
| Mouse | Aim |
| LMB | Melee Attack |
| Space | Dodge Dash |
| Q | Void Grenade |
| E | Repair Pulse |
| B | Build Mode |
| ENTER | Confirm |

## Architecture

```
src/
├── main.rs              # App builder, GameState machine
├── components.rs        # All ECS components & events (single source of truth)
└── plugins/
    ├── world.rs         # Tilemap gen, camera, resource nodes
    ├── player.rs        # Movement, facing, ability cooldowns
    ├── combat.rs        # Hit detection, damage, death
    ├── structures.rs    # Placement, durability, turret AI
    ├── enemies.rs       # Spawning, AI, adaptation tracking
    ├── colony.rs        # NPC roles, hunger, morale
    ├── progression.rs   # EXP, level-up, perks, meta-save
    ├── phase.rs         # Day/Night timer, wave spawner
    ├── audio.rs         # Adaptive stems, SFX
    └── ui.rs            # HUD, modals, menus
```

## GDD

Lihat `void-architect-gdd.docx` untuk game design document lengkap.
