# VOID ARCHITECT — PROGRESS.md
> File ini dibaca Claude di awal setiap sesi untuk memahami status proyek.
> Update setelah setiap task selesai.

---

## 📊 Status Ringkas

```
Versi aktif   : v0.1 MVP
Sprint aktif  : S3 — Progression & Boss (SELESAI) → lanjut S4
Minggu        : 6 / 8
Progress MVP  : 39 / 52 tasks selesai (75%)
Terakhir update: 2026-04-13
```

---

## 🎯 Fokus Sesi Ini

```
Task aktif    : -
Target hari ini: Mulai S4-01 (Full HUD)
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

### Sprint 3 — Progression & Boss

- [x] **S3-01** — EXP system — kill grants via EnemyDied event (×2 saat KillChain aktif), wave bonus +30 EXP tiap Night→Day
- [x] **S3-02** — Level-up pause — LevelUpState resource (flag-based, bukan TimeScale), 3-perk modal [1][2][3], resume on confirm
- [x] **S3-03** — 12 perk implementations (6 Builder, 6 Warrior) — marker components + runtime timers (BerserkerState, KillChainState, TurretOverloadTimer, VoidBurn)
- [x] **S3-04** — Adaptation tracker — update_strategy_tracker tiap akhir malam, wave composition inject VoidCrawler (turret_kill>60%), Stalker extra (npc_kill>50%), stationary_time tracking
- [x] **S3-05** — Boss 1: Swarm Lord — spawn_swarm_lord() + 3 RiftHive di posisi fixed, boss immune sampai semua hive destroyed, Phase 2 saat HP<50%, swarm_lord_ai spawn drone/stalker tiap 4s
- [x] **S3-06** — Void Core entity — spawn_void_core() 500HP di (0,0), monitor_void_core kirim VoidCoreDamaged event, HUD display
- [x] **S3-07** — Win/Lose conditions — check_win_condition (day > 10), check_lose_condition (player dead atau Void Core dead)
- [x] **S3-08** — Void Shards earn — compute_run_end() hitung floor(days + bosses×5), tambah ke MetaProgress, transisi ke MetaScreen
- [x] **S3-09** — Meta save system — dirs::data_local_dir()/VoidArchitect/void_architect_meta.json, atomic write (.tmp → rename), load saat Startup
- [x] **S3-10** — Architect's Sanctum — 5 unlock via SanctumPurchaseEvent, cost validation, instant save setelah purchase, MetaScreen UI dengan shard counter

> **Catatan S3**: Diimplementasi dalam satu sesi di 4 file:
> - `plugins/progression.rs` — 731 baris. S3-01 s/d S3-03, S3-06 s/d S3-10.
>   LevelUpState resource menggantikan TimeScale (tidak ada di Bevy 0.14 built-in).
>   PerkId enum flat untuk 12 perk, SimpleRng untuk pick_three_perks().
>   Atomic save: tulis ke .tmp, rename ke final (mencegah korupsi saat exit paksa).
> - `plugins/enemies.rs` — 691 baris. S3-04 + S3-05.
>   SwarmLord + RiftHive component baru. Boss immune mechanic via hives_alive counter.
>   VoidCrawler sekarang bisa di-inject oleh adaptation tracker (wave 3+).
>   swarm_lord_ai dipisah dari enemy_ai_system (punya query berbeda — ADR-15 tetap terjaga).
> - `plugins/ui.rs` — 530 baris. Level-up modal (Display::None/Flex toggle), MetaScreen
>   dengan Sanctum grid, run-end overlay, Void Core HUD.
> - `src/main.rs` — bersih, semua resource/event diregister di plugin masing-masing.
>
> **ADR baru**: ADR-18 (LevelUpState sebagai pause pengganti TimeScale),
> ADR-19 (Atomic save via .tmp rename).

---

## 🔄 IN PROGRESS

> Kosong — Sprint 3 SELESAI. Siap mulai S4.

---

## 📋 BACKLOG MVP

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

> **Catatan untuk S4**: S4-02 (main menu) dan S4-03/S4-04 (run-end + sanctum UI) sudah
> ada implementasi dasar di ui.rs dari S3. S4 tinggal polish dan lengkapi.
> S4-01 (Full HUD) perlu tambah wave dot indicator dan adapt alert box (belum ada di S3).
> S4-07 audio.rs masih stub — perlu implementasi penuh.

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

### ADR-18: Level-Up Pause — LevelUpState Resource, Bukan TimeScale
**Keputusan:** Game "pause" saat level-up menggunakan `LevelUpState.is_active` flag resource,
bukan TimeScale (yang tidak ada di Bevy 0.14 built-in).  
**Alasan:** Bevy 0.14 tidak punya TimeScale global. Alternatif paling bersih: UI system cek
`LevelUpState.is_active` dan blokir gameplay input. Enemy AI dan phase timer tidak perlu
dihentikan — interaksi minimal selama modal terbuka (durasi pendek).  
**Ditolak:** TimeScale custom (butuh modifikasi Time resource yang risky), `run_if` condition
di semua system (terlalu banyak boilerplate).

### ADR-19: Atomic Save — Tulis .tmp, Rename ke Final
**Keputusan:** Meta save ditulis ke file `.tmp` terlebih dahulu, baru di-rename ke path final.  
**Alasan:** Jika proses crash di tengah penulisan file langsung ke path final, file save bisa
korup dan tidak bisa di-parse. Dengan atomic rename, file lama tetap valid sampai file baru
selesai ditulis sempurna.  
**Ditolak:** Overwrite langsung (risiko korupsi), backup file terpisah (overkill untuk MVP).

---

## 🚧 BLOCKER & CATATAN

> Tidak ada blocker aktif. Sprint 4 bisa dimulai.
>
> **Catatan untuk S4**:
> - S4-02 (main menu) dan S4-03/S4-04 (run-end + sanctum UI) sudah ada implementasi
>   dasar di ui.rs dari S3. S4 tinggal polish, animasi bg, dan complete.
> - S4-01 (Full HUD) perlu tambah wave dot indicator dan adapt alert box — belum ada di S3.
> - S4-07 (adaptive audio) — audio.rs masih stub penuh. Perlu implementasi lengkap.
> - Integrasi perk ke combat.rs dan structures.rs belum selesai:
>   - structures.rs perlu cek PerkIronFrame (wall HP), PerkEfficientBuilder (cost discount),
>     PerkChainTurret (chain shot), PerkArchitectsBastion (+50HP pada placement),
>     TurretOverloadTimer (2× fire rate).
>   - combat.rs perlu cek PerkVoidResonance (apply VoidBurn), PerkArchitectsBastion
>     (50HP shield buffer sebelum damage ke struktur tier 2+).
>   - player.rs perlu cek BerserkerState (3× damage multiplier saat melee),
>     PerkSingularity (pull enemies sebelum void explosion).
>   → Integrasi ini paling natural dikerjakan di S4-10 (balance pass) setelah S4-09 playtest.

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

[2026-04-13] — Sprint S3 SELESAI — S3-01 s/d S3-10 selesai (satu sesi).

               File yang dibuat/dimodifikasi:
               - plugins/progression.rs — ditulis ulang penuh → 731 baris.
                 S3-01: exp_gain_from_kills + exp_wave_bonus (+30 EXP tiap wave clear).
                 S3-02: LevelUpState resource, modal [1][2][3], pick_three_perks() SimpleRng.
                 S3-03: 12 perk via apply_perk() + marker components + runtime state
                        (BerserkerState, KillChainState, TurretOverloadTimer, VoidBurn).
                 S3-06: spawn_void_core() 500HP, monitor_void_core event.
                 S3-07: check_win_condition (day>10) + check_lose_condition (dead).
                 S3-08: compute_run_end() hitung shards = days + bosses×5.
                 S3-09: atomic save via dirs::data_local_dir() + .tmp rename.
                 S3-10: SanctumPurchaseEvent system, 5 unlock, instant save.
               - plugins/enemies.rs — ditulis ulang penuh → 691 baris.
                 S3-04: update_strategy_tracker, VoidCrawler injection (turret>60%),
                        Stalker extra (npc_kill>50%), stationary_time tracking.
                 S3-05: spawn_swarm_lord() + 3 RiftHive, immune mechanic, Phase 2 (<50% HP),
                        swarm_lord_ai (spawn drone/stalker tiap 4s), check_boss_death.
               - plugins/ui.rs — ditulis ulang penuh → 530 baris.
                 Level-up modal (Display::None/Flex), 3 perk card, input [1][2][3].
                 MetaScreen: Sanctum grid 5 unlocks, shard counter, feedback message.
                 Void Core HUD, run-end overlay dengan stats + [ENTER] continue.
               - src/main.rs — minor cleanup, semua resource di plugin masing-masing.

               Keputusan arsitektur baru:
               - ADR-18: LevelUpState resource sebagai "pause" pengganti TimeScale.
               - ADR-19: Atomic save (.tmp → rename) mencegah korupsi file save.
               - swarm_lord_ai dipisah dari enemy_ai_system (SwarmLord punya query
                 berbeda karena butuh VoidCore position) — konsisten dengan ADR-15.

               Catatan integrasi untuk S4:
               - Perk effects di structures.rs (IronFrame, EfficientBuilder, ChainTurret,
                 ArchitectsBastion, TurretOverload) belum terhubung — dikerjakan di S4.
               - combat.rs perlu cek PerkVoidResonance + BerserkerState — dikerjakan di S4.
               - player.rs perlu cek PerkSingularity (pull enemies) — dikerjakan di S4.

               Sprint 3 COMPLETE. Semua 10 task done.
               Sprint berikutnya: S4 — Polish & Ship (Minggu 7–8).
```

---

## 🔢 STATISTIK

```
Total tasks MVP    : 52
Selesai            : 39 (75%)
In progress        : 0
Belum dimulai      : 13

Hari kerja estimasi: ~42 hari
Hari kerja terpakai: ~6 hari (S0 + S1 + S2 + S3 lengkap)
Sprint berikutnya  : S4 — Polish & Ship (S4-01 s/d S4-12)
```
