# Work plan for `pseudo3d`

Derived from [ARCHITECTURE.md](ARCHITECTURE.md) (tree at `33767305be1149970f2c64849e5d06ecbfdda770`). Does not invent modules or bugs beyond that review.

## Goal / current status

**Goal:** crate `pseudo3d` builds on current stable Rust **and** stays alive after startup: window, raycast frame, optional audio, controls.

**Now:**

- **Build passes** on Rust ≥ 1.85 (lockfile pulls `wayland-protocols` with edition 2024). Rust 1.83 does not compile.
- **Tests are empty** (`cargo test` — 0 tests).
- **Runtime does not stay up:** the process dies before the first frame. See [issue #1](https://github.com/wismh/underlife/issues/1).
  1. No ALSA/cpal device: `AudioEngine::new(...).expect("init audio engine")` in `App::new`.
  2. With a dummy PCM: window + GL are created, then `set_cursor_grab(Locked).expect("capture mouse")`.

Priorities below. Each item: why, where in the code, done-when.

---

## P0 — ALSA and `CursorGrabMode::Locked` must not be fatal

**Why.** The binary dies in CI, on VNC/Xvfb, and on a machine with no sound card. That blocks any frame check. Confirmed by a run of this tree; ticket: [issue #1](https://github.com/wismh/underlife/issues/1).

**Where:**

- `src/engine/app.rs` — `App::new`, ~line 47: `.expect("init audio engine")`.
- `src/engine/app.rs` — `capture_mouse`, ~lines 89–93: `.expect("capture mouse")`; same path from `resumed` and from `WindowEvent::Focused(true)`.
- `src/audio/engine.rs` — `AudioEngine::new` already returns `Result<_, AudioError>`; the seam is the call from `App`, not a missing `Result` in the kira wrapper.
- `src/engine/window.rs` — neighboring `.expect`s on swap interval / window create (not in #1, same policy).

**Done when:**

- No sound device → process **does not** panic; the game starts without music/SFX (log instead of `expect`).
- `Locked` unsupported → fall back to `Confined`, then no grab; cursor may stay visible; **a frame is drawn**.
- Repro from #1 (`./target/release/pseudo3d` with no `/dev/snd`; dummy PCM + `xvfb-run` / TigerVNC) no longer yields `thread 'main' panicked at src/engine/app.rs:47` / `:93`.

---

## P1 — Single camera source of truth (FOV)

**Why.** `Player` computes `plane` from `plane_scale` (`assets/configs/player.toml`, currently `0.66`), but the GPU never sees it. `assets/shaders/raycast.frag` has the literal `* 0.66`. Changing the TOML does not change on-screen FOV. Confirmed data-path split, not a hypothesis. See [ARCHITECTURE.md](ARCHITECTURE.md) §4.

**Where:**

- `src/game/player.rs` — fields `plane`, `plane_scale`; `rotate` updates `plane`.
- `src/render/api.rs` — `RaycastScene` has `player_pos` / `player_dir` / `view_bob`, **no** plane.
- `src/engine/app.rs` — `render()` builds `RaycastScene` without `self.player.plane`.
- `src/render/backend/opengl/mod.rs` — uniforms `u_player_pos` / `u_player_dir`, no plane.
- `assets/shaders/raycast.frag` — `vec2 plane = vec2(-dir.y, dir.x) * 0.66;`.

**Done when:**

- `plane` (or `plane_scale`) is on `RaycastScene` and in a shader uniform.
- The `0.66` literal in the frag shader is **gone**.
- Changing `plane_scale` in `player.toml` changes visible FOV without editing the shader.

---

## P2 — Split `App`: host vs session

**Why.** `App` is window, loop, input, audio, player, and demo UIDs. The next map or entity will accrete more fields in `engine::app`. The engine is not a host; it *is* the game. See [ARCHITECTURE.md](ARCHITECTURE.md) §2, §7.

**Where:**

- `src/engine/app.rs` — WASD / arrows / Escape, `map::DEMO`, `texture::{BRICK,FLOOR,SKY}`, `sound_preset::{MUSIC,FOOTSTEPS}`, `upload_gpu_resources`, `update`/`render`.
- `src/engine/mod.rs` — re-exports `run`.
- `src/game/` — currently only `Player`, `HeadBob`, `PlayerConfig`; content bindings should move here (or into a dedicated demo module).
- There is **no** `src/input/` — do not invent it as already existing; it appears only if this item adds it.

**Done when:**

- Host (engine): event loop, `WindowContext`, timing, input buffer, `request_redraw` / present.
- Session / demo: which map, which textures, which loops, WASD → `Player::move_relative`.
- `engine::app` no longer imports concrete `texture::BRICK` / `map::DEMO` as the entire “game”.
- Demo behavior (movement, bob, footsteps, music) is preserved; this is a refactor, not a new game.

---

## P3 — `load_all` → `Result`; resources must not know render types; presets use `SoundUid`

**Why.** Load panics on the first bad file; `TypedStore::get` panics on a missing UID. `ConfigAsset::post_fx` pulls `render` types into the asset layer. Preset `clip = "footsteps"` resolves by string (`sound_by_name`), not UID. That breaks layering and gets worse with more assets. See [ARCHITECTURE.md](ARCHITECTURE.md) §2, §5, §6, §9.

**Where:**

- `src/resources/manager.rs` — `ResourceManager::load_all()` is currently `-> Self` with `.expect` on every file; `TypedStore::get` — `panic!("missing resource uid")`.
- `src/resources/types/config.rs` — `post_fx() -> Result<PostFxSettings, _>` imports `crate::render`.
- `src/engine/app.rs` — `resources.config(config::POSTPROCESS).post_fx().expect(...)`.
- `src/audio/presets.rs` — `resources.sound_by_name(&asset.clip)`.
- `src/resources/types/sound_preset.rs` — field `clip: String`.
- `assets/configs/sounds/footsteps.toml`, `music.toml` — `clip = "footsteps"` / `"music"`.
- `src/resources/manager.rs` — `sound_by_name`.

**Done when:**

- `ResourceManager::load_all() -> Result<Self, AssetError>` (or equivalent); `App` / `run` handle the error without a hidden panic in the loader.
- Parsing `PostFxSettings` lives in `render` or `game` from a generic TOML table, not inside `ConfigAsset` as knowledge of vignette.
- Presets reference `SoundUid` (or a stable ID from the manifest), not a clip name string; `sound_by_name` is not required for this path.

---

## P4 — Collapse or finish stubs; `GlTexture` Drop

**Why.** `MixerState` is empty hooks. `RenderBackend` exists; post does not use it. `GlTexture` has no `Drop`; `OpenGlBackend::drop` deletes program + VAO, not textures; re-upload leaks the handle. See [ARCHITECTURE.md](ARCHITECTURE.md) §2, §4.

**Where:**

- `src/audio/mixer.rs` — `set_ducking` / `set_reverb_send` / `set_eq_low` empty; `AudioEngine` holds `MixerState`.
- `src/audio/volume.rs` — unused `linear_to_kira`, `combine_linear`.
- `src/audio/spatial.rs` — `let _ = linear_to_decibels(volume_linear)`.
- `src/render/api.rs` — trait `RenderBackend`.
- `src/render/pipeline.rs` — hardcoded `OpenGlBackend` + `OpenGlPostFx`.
- `src/render/postprocess/api.rs` — `PostProcessBackend` with `&glow::Context`.
- `src/render/backend/opengl/mod.rs` — `struct GlTexture(glow::NativeTexture)`; backend `Drop` without deleting textures.
- `src/render/raycast.rs` — `set_*` replaces `Option<B::Texture>` without freeing the previous one.

**Done when (pick one path, deliberately):**

- **Collapse:** remove or stop exporting the dead mixer/volume helpers; do not pretend multi-backend while post is glued to glow; **or**
- **Finish:** mixer actually does something; post sits behind the same seam as raycast.
- And in either case: `GlTexture` (or equivalent) frees `NativeTexture` on drop; a second `set_wall_texture` does not leave an orphan GL name.

---

## P5 — Tests, `rust-version`, README, Git LFS

**Why.** The only pure code (map parse, collision, bob, FOV path after P1) is untested. No `rust-version` / README — 1.83 cannot even parse the lockfile. PNG/OGG/MP3 are Git LFS; a clone with `GIT_LFS_SKIP_SMUDGE` yields pointer files. See [ARCHITECTURE.md](ARCHITECTURE.md) §8–§9.

**Where:**

- `src/resources/types/map.rs` — `parse_row`, `MapAsset::load`, `is_wall`.
- `src/game/player.rs` — `move_relative`.
- `src/game/head_bob.rs` — `HeadBob::update`.
- After P1 — FOV path (scene → uniform → shader) at least a unit test of `RaycastScene` assembly / plane constant.
- `Cargo.toml` — no `rust-version`.
- Repo root README — short pointer at these docs.
- `.gitattributes` — LFS for `*.png` `*.ogg` `*.mp3`.
- `src/resources/paths.rs` — `PSEUDO3D_ASSETS`.

**Done when:**

- There are `#[test]`s for parsing `demo.map` (or a fixture), for `is_wall` / `move_relative` (does not walk through `#`), for bob update (offset changes when `bob_speed > 0`).
- `Cargo.toml` has `rust-version = "1.85"` (or the actual lockfile minimum).
- README (short) points at [ARCHITECTURE.md](ARCHITECTURE.md) and this file, build (Rust ≥ 1.85), and `git lfs pull`.
- Docs or README note: without LFS smudge, assets are pointer files; run `git lfs pull`.

---

## Order

P0 blocks visual verification. P1 is local and can run in parallel with P0. P2 is easier after P0 (a live window exists). P3 does not depend on GL. P4 is cleanup after the layers stabilize. P5 can start with map/player units immediately; `rust-version`/README can ship with any of these PRs.

As-is detail: [ARCHITECTURE.md](ARCHITECTURE.md).
