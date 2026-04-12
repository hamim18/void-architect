# VOID ARCHITECT — PROGRESS.md
> File ini dibaca Claude di awal setiap sesi untuk memahami status proyek.
> Update setelah setiap task selesai.

---

## 📊 Status Ringkas

```
Versi aktif   : v0.1 MVP
Sprint aktif  : S1 — Combat & Structures
Minggu        : 2 / 8
Progress MVP  : 8 / 52 tasks selesai (15%)
Terakhir update: 2026-04-12
```

---

## 🎯 Fokus Sesi Ini

```
Task aktif    : -
Target hari ini: S1-01 — Melee attack
Blocker       : -
```

---

## ✅ SELESAI

### Sprint 0 — Foundation

- [x] **S0-01** — Cargo workspace + Bevy app scaffold (GameState: MainMenu, InRun, MetaScreen)
- [x] **S0-02** — Core ECS components: Position, Health, Player, Enemy, Structure, NPC, semua Events
- [x] **S0-03** — Tile map Ashlands (Perlin noise, 40×30 tiles, 32px), camera follow
- [x] **S0-04** — Player controller — LMB click-to-move, mouse facing, KinematicVelocityBased
- [x] **S0-05** — Phase timer — Day 180s / Night 120s, PhaseChanged event dispatch
- [x] **S0-06** — Resource system — Stone/Scrap/Food/Crystal, can_afford/spend API
- [x] **S0-07** — Resource node spawning — scatter map, auto-collect on proximity
- [x] **S0-08** — GitHub CI — cargo fmt + clippy + build + test on push

---

## 🔄 IN PROGRESS

> Kosong — siap mulai S1.

---

## 📋 BACKLOG MVP

### Sprint 1 — Combat & Structures (Minggu 3–4)

- [ ] **S1-01** — Melee attack — arc sweep 45° cone, hit detection, flash effect, 0.4s CD `1d`
- [ ] **S1-02** — Dodge dash — short teleport, 8-frame iframes, 3s CD `0.5d`
- [ ] **S1-03** — Void Grenade — projectile, AOE 80px, stagger, 12s CD `1d`
- [ ] **S1-04** — Repair Pulse — heal nearby structures 20HP, AOE 100px, 20s CD `0.5d`
- [ ] **S1-05** — Wall structure — grid placement, 100HP, repair mechanic `1d`
- [ ] **S1-06** — Turret structure — auto-target, projectile, 150px range, 1.5s fire rate `1.5d`
- [ ] **S1-07** — Farm structure — 3 Food/day on PhaseChanged(Day) `0.5d`
- [ ] **S1-08** — Build mode — hotbar selection, ghost preview, snap placement, resource deduction `1d`
- [ ] **S1-09** — Enemy: Void Drone — A* pathfinding, chase player `1d`
- [ ] **S1-10** — Enemy: Breacher — ignore player, target nearest wall `0.5d`
- [ ] **S1-11** — Enemy: Rift Stalker — fast, flank, prefer NPC targets `1d`
- [ ] **S1-12** — Wave spawn system — edge spawning, count scaling (+2/wave), Night trigger `1d`
- [ ] **S1-13** — Death system — entity despawn, loot drop, EXP grant, particle burst `0.5d`

### Sprint 2 — Colony Systems (Minggu 5)

- [ ] **S2-01** — NPC entity — spawn, wander AI, role assignment, start 2 NPCs `2d`
- [ ] **S2-02** — Farmer role — assign to Farm, +2 Food/day `0.5d`
- [ ] **S2-03** — Builder role — reduce construction time 30% `0.5d`
- [ ] **S2-04** — Guard role — patrol zone, attack nearby enemies, 40HP `1d`
- [ ] **S2-05** — Hunger system — 1–2 Food/NPC/day deduction per day cycle `1d`
- [ ] **S2-06** — Starvation: Day 1 = -10 morale, Day 2 = 1 NPC deserts, Day 3 = 1 NPC dies `0.5d`
- [ ] **S2-07** — Morale system — 0–100, event hooks, efficiency modifiers below 30 `1d`
- [ ] **S2-08** — House structure — 4 NPC cap per house, block rescue if full `0.5d`
- [ ] **S2-09** — NPC rescue event — day-phase trigger, [R/Skip] prompt, 5s timeout `0.5d`

### Sprint 3 — Progression & Boss (Minggu 6)

- [ ] **S3-01** — EXP system — kill grants, wave bonus, escalating threshold (×1.4 per level) `0.5d`
- [ ] **S3-02** — Level-up pause — TimeScale(0), 3-perk modal, resume on confirm `1d`
- [ ] **S3-03** — 12 perk implementations (6 Builder, 6 Warrior) `2d`
- [ ] **S3-04** — Adaptation tracker — StrategyTracker resource, wave flag injection `1d`
- [ ] **S3-05** — Boss 1: Swarm Lord — AI, 3 Rift Hive spawners, Phase 2 `1.5d`
- [ ] **S3-06** — Void Core entity — 500HP, damage events, screen shake, lose condition `0.5d`
- [ ] **S3-07** — Win/Lose conditions — Day 10 survive = win; Core=0 / HP=0 = lose `0.5d`
- [ ] **S3-08** — Void Shards earn — floor(days * 1.0 + boss_kills * 5) `0.5d`
- [ ] **S3-09** — Meta save system — serde_json serialize MetaProgress to disk `0.5d`
- [ ] **S3-10** — Architect's Sanctum — 5 MVP unlocks, spend Void Shards `0.5d`

### Sprint 4 — Polish & Ship (Minggu 7–8)

- [ ] **S4-01** — Full HUD — all panels, bars, wave dots, adapt alert `2d`
- [ ] **S4-02** — Main menu — title, New Run, Sanctum, Quit, animated bg `1d`
- [ ] **S4-03** — Run-end screen — stats, Void Shards earned, continue button `0.5d`
- [ ] **S4-04** — Architect's Sanctum UI — unlock grid, shard balance `0.5d`
- [ ] **S4-05** — Particle system — hit sparks, death burst, void glow `1d`
- [ ] **S4-06** — Screen shake — player hit, Void Core damage, explosion `0.5d`
- [ ] **S4-07** — Adaptive audio — 4 stems, crossfade logic, state machine `1d`
- [ ] **S4-08** — SFX — melee, death, turret, explosion, UI, phase transition `1d`
- [ ] **S4-09** — Playtest session 1 — full run Day 1→10, document bugs `1d`
- [ ] **S4-10** — Balance pass — wave scaling, economy, hunger tension `1d`
- [ ] **S4-11** — Bug fix sprint — semua P1 dari playtest `1d`
- [ ] **S4-12** — Release build — cargo build --release, upload itch.io `0.5d`

---

## 🗂️ POST-MVP BACKLOG

> Jangan dikerjakan selama Sprint S0–S4.

- [ ] Biome 2 — Void Marsh
- [ ] Biome 3 — Iron Ruins
- [ ] Biome 4 — Rift Core
- [ ] Boss 2 — Void Walker
- [ ] Boss 3 — Hollow King
- [ ] Boss 4 — Architect's Echo (Final Boss)
- [ ] T3 structures: Tesla Tower, Barrier Shield, Storehouse
- [ ] T4 structures: Void Cannon, Rune Gate
- [ ] NPC: Healer
- [ ] NPC: Scavenger
- [ ] Codex system
- [ ] Character: The Wraith
- [ ] Pixel art sprites (ganti geometric MVP)
- [ ] Biome music pack per biome
- [ ] New Game+ / Void Surge
- [ ] Full 40 perks
- [ ] Steam integration

---

## 📐 ADR — Architecture Decision Records

### ADR-01: Engine — Bevy (Rust)
**Keputusan:** Pakai Bevy ECS untuk game engine.  
**Alasan:** ECS ideal untuk ratusan entitas. Rust = performa tinggi. Hamim familiar dengan Rust.  
**Ditolak:** Godot, Unity.

### ADR-02: Audio Generation — Python + numpy
**Keputusan:** Semua audio di-generate prosedural via Python (game-sound-forge).  
**Alasan:** Zero dependency external tools. Mobile-friendly. Reproducible.  
**Ditolak:** Manual recording, asset packs.

### ADR-03: MVP Visual — Geometric Primitives
**Keputusan:** MVP pakai Bevy geometric rendering, bukan sprite sheet.  
**Alasan:** Eliminasi bottleneck asset art. Fokus ke sistem dulu.  
**Ditolak:** Langsung pixel art (terlalu lambat untuk solo dev).

### ADR-04: Save Format — serde_json
**Keputusan:** MetaProgress di-serialize ke JSON via serde_json.  
**Alasan:** Human-readable untuk debugging, mudah di-migrate.  
**Ditolak:** SQLite (overkill), RON (kurang familiar untuk save games).

### ADR-05: Player Physics — KinematicVelocityBased
**Keputusan:** Player pakai `RigidBody::KinematicVelocityBased`, bukan `Dynamic`.  
**Alasan:** `Dynamic` rigidbody **overwrite** `Transform.rotation` setiap physics tick,
sehingga rotasi manual dari mouse facing langsung di-cancel rapier. Kinematic body
tidak apply torque/gravity, sehingga kita bebas set rotation manual via `player_mouse_facing`
tanpa konflik. Velocity tetap bisa di-set langsung (`vel.linvel`).  
**Ditolak:** `Dynamic` + `LockedAxes::ROTATION_LOCKED` (tetap konflik), `Dynamic` +
`angular_damping: 100.0` (rapier masih override di physics step).

### ADR-06: Player Movement — LMB Click-to-Move
**Keputusan:** Player bergerak dengan klik kiri (LMB) ke titik tujuan. Facing selalu ke arah cursor.  
**Alasan:** Cocok dengan genre top-down colony defense. LMB untuk move, LMB juga akan
double sebagai attack (attack saat target adalah enemy — diimplementasi S1-01).  
**Ditolak:** WASD (kurang intuitif untuk build+move hybrid gameplay).

### ADR-07: Bevy Color API — srgb bukan rgb
**Keputusan:** Semua color literal pakai `Color::srgb()`.  
**Alasan:** `Color::rgb()` deprecated di Bevy 0.14. `srgb` adalah API yang benar.  
**Catatan:** Semua file sudah di-migrate. Jangan pakai `Color::rgb()` lagi.

### ADR-08: Camera Follow — XY only, Z tidak disentuh
**Keputusan:** `camera_follow_player` lerp hanya di XY. `translation.z` kamera tidak diubah.  
**Alasan:** Bug kritis ditemukan: lerp ke `target.extend(999.9)` menyebabkan kamera
makin frame makin jauh di Z → semua tile (z = -10) keluar dari near clip plane dan
tidak ter-render. Camera2dBundle default z = 999.9, biarkan di sana.  
**Fix:** Set `ctf.translation.x` dan `.y` terpisah, `.z` tidak disentuh.

### ADR-09: Tile Z-Layer — z = -10.0
**Keputusan:** Tile di-spawn di `z = -10.0`, gameplay entity di `z = 0.5–2.0`.  
**Alasan:** Memberi ruang layer yang jelas. Camera (z ≈ 999.9) melihat semua layer
dari -1000 sampai 1000 di Bevy 2D default projection.  
**Sebelumnya:** Tile di z = -1.0 (terlalu dekat dengan gameplay entity, berisiko z-fighting).

### ADR-10: Tidak Ada Velocity Component di components.rs
**Keputusan:** Tidak ada `Velocity` custom di `components.rs`. Plugin yang butuh velocity
query `bevy_rapier2d::prelude::Velocity` langsung.  
**Alasan:** Konflik nama antara custom `Velocity` dan rapier `Velocity` menyebabkan
`E0659: ambiguous name` saat keduanya di-glob import. Solusi: hapus custom Velocity,
pakai rapier punya saja.

### ADR-11: Cargo.toml — Tanpa dynamic_linking
**Keputusan:** Feature `dynamic_linking` tidak dipakai di Cargo.toml.  
**Alasan:** Menyebabkan crash/linker error di beberapa Linux setup (terutama saat compile
dari mobile/remote environment). Sudah dihapus dari dependencies.  
**Trade-off:** Build lebih lambat saat development. Bisa diaktifkan kembali secara lokal
dengan `--features bevy/dynamic_linking` jika compile time jadi masalah.

---

## 🚧 BLOCKER & CATATAN

> Kosong saat ini.

---

## 📝 SESSION LOG

```
[2026-04-11] — Sesi pertama. Semua dokumen proyek selesai dibuat.
               Proyek siap dimulai dari S0-01.

[2026-04-12] — Sprint S0 selesai semua (S0-01 s/d S0-08).
               File yang dihasilkan: Cargo.toml, main.rs, components.rs,
               plugins/{world, player, phase, progression, ui, combat,
               colony, enemies, structures, audio}.rs, .github/workflows/ci.yml,
               assets/data/biomes.ron, .gitignore, README.md.

               Bug yang ditemukan dan difix selama S0:
               1. AppExit bukan value → AppExit::Success
               2. Color::rgb deprecated → Color::srgb (18 file)
               3. Velocity ambiguous → hapus custom Velocity, pakai rapier langsung
               4. dynamic_linking feature → dihapus (crash di beberapa setup)
               5. Player nyangkut ke bawah → RigidBody::Dynamic konflik dengan
                  manual rotation → ganti ke KinematicVelocityBased (ADR-05)
               6. Tilemap hilang setelah beberapa frame → camera lerp ke z=999.9
                  tiap frame, tiles keluar clip plane → fix: lerp XY only (ADR-08)
               7. Movement pakai RMB → diubah ke LMB sesuai desain (ADR-06)
               8. Duplicate definitions di player.rs → str_replace append bukan
                  replace → rewrite file penuh sebagai workaround

               Status akhir S0: compile clean, tilemap render, player bisa
               gerak LMB click-to-move, facing ke cursor, resource node collect.
               Siap lanjut S1.
```

---

## 🔢 STATISTIK

```
Total tasks MVP    : 52
Selesai            : 8  (15%)
In progress        : 0
Belum dimulai      : 44

Hari kerja estimasi: ~42 hari
Hari kerja terpakai: ~2 hari (S0)
Sprint berikutnya  : S1 — Combat & Structures (13 tasks, est. 10 hari)
```
