# VOID ARCHITECT — PROGRESS.md
> File ini dibaca Claude di awal setiap sesi untuk memahami status proyek.
> Update setelah setiap task selesai.

---

## 📊 Status Ringkas

```
Versi aktif   : v0.1 MVP
Sprint aktif  : S2 — Colony Systems (SELESAI) → lanjut S3
Minggu        : 5 / 8
Progress MVP  : 29 / 52 tasks selesai (56%)
Terakhir update: 2026-04-12
```

---

## 🎯 Fokus Sesi Ini

```
Task aktif    : -
Target hari ini: Mulai S3-01 (EXP system)
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

### Sprint 1 — Combat & Structures

- [x] **S1-01** — Melee attack — arc sweep 45° cone (cos 22.5°), [F] atau LMB ke enemy, flash effect, 0.4s CD
- [x] **S1-02** — Dodge dash — teleport ke arah facing 90px, 8-frame iframes (Invincible component), 3s CD, [Space]
- [x] **S1-03** — Void Grenade — projectile terbang ke cursor, AOE 80px explosion, 2s lifetime, 12s CD, [E]
- [x] **S1-04** — Repair Pulse — heal struktur dalam 100px +20HP, 20s CD, [R]
- [x] **S1-05** — Wall structure — RigidBody::Fixed collider, 100HP, snap ke grid 32px, repair mechanic via RepairPulse
- [x] **S1-06** — Turret structure — auto-target enemy terdekat 150px, homing projectile 15dmg 1.5s fire rate, barrel berputar
- [x] **S1-07** — Farm structure — produksi 3 Food/day saat PhaseChanged(Day), HP-based output
- [x] **S1-08** — Build mode — [B] toggle, ghost preview (hijau=affordable/merah=tidak), LMB=place, ESC=cancel, [W/T/G/H] pilih jenis
- [x] **S1-09** — Enemy: Void Drone — AI chase player, drone_waypoint() hindari wall (36px clearance check)
- [x] **S1-10** — Enemy: Breacher — target WallMarker terdekat setiap 0.5s, ignore player, attack range 28px
- [x] **S1-11** — Enemy: Rift Stalker — cepat 170px/s, FlankPhase (Approach 60° → Strike +25% speed), prefer Npc target
- [x] **S1-12** — Wave spawn system — PhaseChanged(Night) trigger, queue bertahap (0.35s→0.12s interval), adaptation: wall_reliance>0.7 → +Breacher
- [x] **S1-13** — Death system — check_enemy_death di combat.rs (despawn + EnemyDied event), EXP grant di progression.rs, loot drop scrap

> **Catatan S1-09~S1-12**: Diimplementasi dalam satu file `plugins/enemies.rs`.
> Semua AI digabung dalam `enemy_ai_system` (satu Query Enemy mutable) untuk
> menghindari Bevy B0001 mutable query conflict — pola yang sama dengan ADR-14 (turret_ai).
> Wave spawner pakai SimpleRng (LCG internal) — tidak butuh dependency `rand` tambahan.

### Sprint 2 — Colony Systems

- [x] **S2-01** — NPC entity — spawn 2 NPC awal (Farmer+Builder), wander AI via SimpleRng, role assignment
- [x] **S2-02** — Farmer role — npc_farmer_assign() ke Farm terdekat, +2 Food/day bonus via hunger_system
- [x] **S2-03** — Builder role — resource BuilderBonus (construction_speed_bonus 0.30), diupdate tiap frame
- [x] **S2-04** — Guard role — patrol zona GUARD_PATROL_RADIUS, chase + attack enemy, 40HP, 1.2s attack CD
- [x] **S2-05** — Hunger system — potong food tiap PhaseChanged(Day), 1 food/NPC (Guard=2), Farmer bonus food
- [x] **S2-06** — Starvation — Day1=−10 morale, Day2=1 NPC deserts (despawn), Day3=1 NPC mati + −10 morale
- [x] **S2-07** — Morale system — 0–100, apply_morale_delta() pub helper, npc_efficiency() modifier, MoraleChanged event
- [x] **S2-08** — House structure — update_house_cap() dari HouseMarker count, max_population=houses×4 (min 2)
- [x] **S2-09** — NPC rescue event — RescueTrigger spawn saat Day, [R] rescue / [N] skip, 5s timeout, cooldown 60s

> **Catatan S2**: Semua 9 task diimplementasi dalam satu sesi di `plugins/colony.rs` (872 baris).
> Guard AI mengikuti pola ADR-15 — satu query Enemy untuk npc_guard_ai menghindari B0001.
> `components.rs` diupdate: `NpcRole` tambah `Default` derive + `#[default] Idle`,
> `ColonyState::default()` population diubah dari 2 → 0 (spawn_starting_npcs yang increment).
> `ui.rs` diupdate: tambah HudColony (pop/morale/food bar) dan HudRescuePrompt (prompt [R/N]).
> Bug fix compile: E0277 NpcRole tidak impl Default → tambah derive. Warnings dibersihkan.

---

## 🔄 IN PROGRESS

> Kosong — Sprint 2 SELESAI. Siap mulai S3.

---

## 📋 BACKLOG MVP

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
**Alasan:** Cocok dengan genre top-down colony defense. LMB ke enemy = attack (S1-01), LMB ke tanah = move.  
**Ditolak:** WASD (kurang intuitif untuk build+move hybrid gameplay).

### ADR-07: Bevy Color API — srgb bukan rgb
**Keputusan:** Semua color literal pakai `Color::srgb()`.  
**Alasan:** `Color::rgb()` deprecated di Bevy 0.14. `srgb` adalah API yang benar.  
**Catatan:** Semua file sudah di-migrate. Jangan pakai `Color::rgb()` lagi.

### ADR-08: Camera Follow — XY only, Z tidak disentuh
**Keputusan:** `camera_follow_player` lerp hanya di XY. `translation.z` kamera tidak diubah.  
**Alasan:** Bug kritis: lerp ke `target.extend(999.9)` menyebabkan kamera makin jauh di Z
→ tile (z = -10) keluar dari near clip plane dan tidak ter-render.

### ADR-09: Tile Z-Layer — z = -10.0
**Keputusan:** Tile di-spawn di `z = -10.0`, gameplay entity di `z = 0.5–2.0`.  
**Alasan:** Layer yang jelas. Camera (z ≈ 999.9) melihat semua layer dari -1000 sampai 1000.

### ADR-10: Tidak Ada Velocity Component di components.rs
**Keputusan:** Tidak ada `Velocity` custom di `components.rs`. Plugin yang butuh velocity
query `bevy_rapier2d::prelude::Velocity` langsung.  
**Alasan:** Konflik nama `E0659: ambiguous name` antara custom dan rapier Velocity.

### ADR-11: Cargo.toml — Tanpa dynamic_linking
**Keputusan:** Feature `dynamic_linking` tidak dipakai di Cargo.toml.  
**Alasan:** Crash/linker error di beberapa Linux setup. Sudah dihapus dari dependencies.

### ADR-12: Build Mode — [B] Toggle, Day-Only
**Keputusan:** Build mode hanya bisa diaktifkan saat Day phase. Placement diblokir saat Night.  
**Alasan:** Sesuai GDD: "Construct and upgrade defensive structures during the day phase."  
Night = defend only. Ini juga mencegah exploit membangun saat wave aktif.

### ADR-14: Multi-Transform Query — ParamSet
**Keputusan:** System yang butuh akses `Transform` dari beberapa kelompok entitas berbeda
wajib menggunakan `ParamSet` jika salah satu query minta `&mut Transform`.  
**Alasan:** Bevy tidak bisa verify secara statik bahwa query-query tersebut disjoint jika
filter `Without<T>` tidak cukup eksplisit. Tanpa `ParamSet`, Bevy panic B0001 saat runtime.
`ParamSet` memaksa hanya satu query aktif per waktu sehingga aliasing `&mut` tidak mungkin terjadi.  
**Contoh:** `turret_ai` — 3 query Transform (turret/barrel/enemy) direfaktor ke `ParamSet<(p0, p1, p2)>`
dengan 4 pass berurutan. Data diangkat ke struct lokal (`TurretData`, `FireData`) antar pass.  
**Ditolak:** Menambah `Without<Enemy>` + `Without<TurretBarrel>` ke semua query (fragile,
mudah break kalau component bertambah di masa depan).

### ADR-15: Enemy AI — Single System (enemy_ai_system)
**Keputusan:** Semua AI enemy (Void Drone, Breacher, Rift Stalker) diimplementasi dalam
satu system `enemy_ai_system` dengan `match enemy.variant`, bukan tiga system terpisah.  
**Alasan:** Tiga system terpisah yang masing-masing punya `Query<(&Transform, &mut Velocity,
&mut EnemyAiState, &Enemy)>` akan trigger B0001 mutable query conflict di Bevy — Bevy tidak
bisa verify bahwa ketiga query tersebut disjoint karena tidak ada filter yang memisahkan
mereka secara eksplisit. Satu system = satu query = zero conflict.  
**Ditolak:** Tiga system terpisah dengan type-filter (tidak ada EnemyType component terpisah
per type — EnemyType ada di dalam Enemy component, sehingga tidak bisa dipakai sebagai filter).

### ADR-16: NPC AI — Guard Terpisah dari Wander System
**Keputusan:** Guard punya system sendiri (`npc_guard_ai`), Idle/Farmer/Builder di `npc_wander_ai`.  
**Alasan:** Guard butuh query `Enemy` untuk attack — kalau digabung di satu system dengan
wander, muncul B0001 conflict karena wander butuh `&mut Velocity` per NPC sementara guard
juga butuh `&mut Velocity` + query Enemy. Dipisah = dua system dengan query berbeda = aman.  
**Ditolak:** Satu mega-system NPC (terlalu kompleks, sulit di-debug per role).

### ADR-17: ColonyState population — Start dari 0, Diincrement oleh spawn_starting_npcs
**Keputusan:** `ColonyState::default()` set `population: 0`. Fungsi `spawn_starting_npcs`
yang increment ke 2 saat `OnEnter(InRun)`.  
**Alasan:** Mencegah double-count antara default value (2) dan increment manual saat spawn.
Jika default = 2 dan spawn juga increment, population akan terhitung 4 padahal hanya 2 NPC.  
**Ditolak:** Default population: 2 tanpa spawn increment (tidak sinkron dengan entity yang actual ada).

---

## 🚧 BLOCKER & CATATAN

> Tidak ada blocker aktif. Sprint 3 bisa dimulai.
>
> **Catatan untuk S3**: `progression.rs` sudah punya skeleton EXP system (exp_gain_from_kills,
> check_level_up) — S3-01 tinggal verifikasi dan tambah wave bonus. S3-02 butuh TimeScale
> yang belum ada di Bevy 0.14 secara built-in — alternatif: pause via custom flag resource
> dan skip Update systems dengan `run_if` condition.

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

[2026-04-12] — Sprint S1 dimulai — S1-01 s/d S1-08 selesai (satu sesi).

               File yang dimodifikasi:
               - plugins/player.rs — Melee (S1-01), Dash (S1-02), Grenade (S1-03),
                 Repair Pulse (S1-04). Input terpusat di handle_player_input.
               - plugins/structures.rs — Wall (S1-05), Turret (S1-06), Farm (S1-07),
                 Build Mode (S1-08). Turret AI + projectile homing. Farm food production.
               - plugins/combat.rs — process_damage_events extended, check_enemy_death
                 (S1-13 partial), spawn_loot, check_void_core_damage.
               - plugins/ui.rs — Extended debug HUD: cooldown display, build mode indicator.
               - plugins/progression.rs — ResourceCost derive Copy (fix const usage).
               - Cargo.toml — Hapus dynamic_linking (ADR-11).

               ADR baru: ADR-12 (Build Mode day-only), ADR-13 (DashTarget resource pattern).

               Yang tersisa di S1: S1-09, S1-10, S1-11, S1-12 (enemy AI + wave spawn).
               S1-13 death system sudah 80% di combat.rs (tinggal particle VFX di S4-05).

[2026-04-12] — Bug fix session — structures.rs.

               Bug 1 (compile error E0255): pub use self::BuildMode dan pub use
               self::BuildSelection → dihapus (tidak perlu re-export dari file yang sama).
               Bug 2 (compile error E0277): .add_systems(OnEnter(InRun), ()) → dihapus.
               Bug 3 (compile warnings): parameter tidak terpakai → dihapus.
               Bug 4 (runtime panic Bevy B0001): turret_ai 3 Query Transform bersamaan
               → refaktor ke ParamSet<(p0, p1, p2)> dengan 4 pass. ADR-14 ditambahkan.

[2026-04-12] — Sprint S1 SELESAI — S1-09, S1-10, S1-11, S1-12 selesai.

               File yang dimodifikasi:
               - plugins/enemies.rs — ditulis ulang penuh dari stub kosong.
                 619 baris. Berisi: wave spawn system (S1-12), Void Drone AI (S1-09),
                 Breacher AI (S1-10), Rift Stalker AI (S1-11), enemy attack system.

               Keputusan arsitektur baru:
               - ADR-15: Semua AI enemy dalam satu enemy_ai_system untuk menghindari
                 B0001 mutable query conflict (sama pattern dengan ADR-14 turret_ai).
               - Wave spawner pakai SimpleRng (LCG internal) — tidak tambah dependency rand.
               - Adaptation dari StrategyTracker sudah terintegrasi di build_wave_composition:
                 wall_reliance > 0.7 → breacher_ratio naik dari 15% ke 35%.
               - Stalker belum spawn di wave 1 (mulai wave 2) sesuai GDD.

               S1-13 (death system) sudah complete dari sesi sebelumnya:
               - check_enemy_death di combat.rs ✓
               - EXP grant via EnemyDied event di progression.rs ✓
               - Loot drop (scrap) di spawn_loot ✓
               - Particle VFX di-defer ke S4-05 (sesuai rencana)

               Sprint 1 COMPLETE. Semua 13 task done.
               Sprint berikutnya: S2 — Colony Systems (Minggu 5).

[2026-04-12] — Sprint S2 SELESAI — S2-01 s/d S2-09 selesai (satu sesi).

               File yang dibuat/dimodifikasi:
               - plugins/colony.rs — ditulis ulang penuh dari stub 19 baris → 872 baris.
                 Berisi: spawn 2 NPC awal, wander AI, Farmer assign, BuilderBonus resource,
                 Guard patrol+attack, hunger system, starvation consequences, morale system,
                 house cap, rescue event [R/N] + 5s timeout.
               - components.rs — NpcRole tambah Default derive + #[default] Idle.
                 ColonyState::default() population: 2 → 0.
               - ui.rs — tambah HudColony (pop/morale/food/starvation display) dan
                 HudRescuePrompt (prompt [R/N] dengan countdown).

               Keputusan arsitektur baru:
               - ADR-16: Guard AI dipisah dari wander system (query berbeda, hindari B0001).
               - ADR-17: ColonyState population start dari 0, diincrement spawn_starting_npcs.
               - SimpleRng (LCG) dipakai ulang di colony.rs — konsisten dengan enemies.rs.
               - BuilderBonus resource pub — bisa dibaca structures.rs di Sprint 3/4.
               - apply_morale_delta() dan npc_efficiency() di-export pub untuk dipakai
                 sistem lain (combat, enemies) saat event morale dari luar colony.

               Bug fix compile:
               - E0277: NpcRole tidak impl Default → tambah derive Default + #[default] Idle.
               - Warning unused_parens di match → hapus parens.
               - Warning unused_mut di morale_daily_events → ganti ResMut → Res, hapus
                 EventWriter yang tidak dipakai di body.

               Sprint 2 COMPLETE. Semua 9 task done.
               Sprint berikutnya: S3 — Progression & Boss (Minggu 6).
```

---

## 🔢 STATISTIK

```
Total tasks MVP    : 52
Selesai            : 29 (56%)
In progress        : 0
Belum dimulai      : 23

Hari kerja estimasi: ~42 hari
Hari kerja terpakai: ~5 hari (S0 + S1 + S2 lengkap)
Sprint berikutnya  : S3 — Progression & Boss (S3-01 s/d S3-10)
```
