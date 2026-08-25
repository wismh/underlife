# План робіт `pseudo3d`

Похідний від [ARCHITECTURE.md](ARCHITECTURE.md) (стан дерева на `33767305be1149970f2c64849e5d06ecbfdda770`). Не вигадує модулів чи багів поза тим оглядом.

## Мета / поточний статус

**Мета:** крейт `pseudo3d` збирається на актуальному stable Rust **і** залишається живим після старту: вікно, кадр рейкасту, опційне аудіо, керування.

**Зараз:**

- Збірка **проходить** на Rust ≥ 1.85 (lockfile тягне `wayland-protocols` з edition 2024). Rust 1.83 — ні.
- Тести **порожні** (`cargo test` — 0 тестів).
- Рантайм **не тримається**: процес падає до першого кадру. Див. [issue #1](https://github.com/wismh/underlife/issues/1).
  1. Без ALSA/cpal-пристрою: `AudioEngine::new(...).expect("init audio engine")` у `App::new`.
  2. З dummy PCM: вікно + GL створюються, далі `set_cursor_grab(Locked).expect("capture mouse")`.

Нижче — пріоритети. Кожен пункт: навіщо, де в коді, коли вважати готовим.

---

## P0 — ALSA і `CursorGrabMode::Locked` не фатальні

**Навіщо.** Зараз бінарник помирає в CI, на VNC/Xvfb і на машині без звукової карти. Це блокує будь-яку перевірку кадру. Підтверджено прогоном; тікет: [issue #1](https://github.com/wismh/underlife/issues/1).

**Де:**

- `src/engine/app.rs` — `App::new`, рядок ~47: `.expect("init audio engine")`.
- `src/engine/app.rs` — `capture_mouse`, рядки ~89–93: `.expect("capture mouse")`; той самий шлях з `resumed` і з `WindowEvent::Focused(true)`.
- `src/audio/engine.rs` — `AudioEngine::new` уже повертає `Result<_, AudioError>`; шов — виклик з `App`, не відсутність `Result` у kira-обгортці.
- `src/engine/window.rs` — сусідні `.expect` на swap interval / створення вікна (не в #1, але тієї ж політики).

**Зроблено, коли:**

- Немає звукового пристрою → процес **не** панікує; гра стартує без музики/SFX (лог замість `expect`).
- `Locked` не підтримується → fallback на `Confined`, потім без grab; курсор можна лишити видимим; **кадр малюється**.
- Репро з #1 (`./target/release/pseudo3d` без `/dev/snd`; dummy PCM + `xvfb-run` / TigerVNC) більше не дає `thread 'main' panicked at src/engine/app.rs:47` / `:93`.

---

## P1 — Одне джерело правди для камери (FOV)

**Навіщо.** `Player` рахує `plane` з `plane_scale` (`assets/configs/player.toml`, зараз `0.66`), але GPU його не бачить. У `assets/shaders/raycast.frag` літерал `* 0.66`. Зміна TOML не змінює FOV на екрані. Це підтверджений розрив, не гіпотеза. Див. [ARCHITECTURE.md](ARCHITECTURE.md) §4.

**Де:**

- `src/game/player.rs` — поля `plane`, `plane_scale`; `rotate` оновлює `plane`.
- `src/render/api.rs` — `RaycastScene` має `player_pos` / `player_dir` / `view_bob`, **немає** plane.
- `src/engine/app.rs` — `render()` збирає `RaycastScene` без `self.player.plane`.
- `src/render/backend/opengl/mod.rs` — uniforms `u_player_pos` / `u_player_dir`, немає plane.
- `assets/shaders/raycast.frag` — `vec2 plane = vec2(-dir.y, dir.x) * 0.66;`.

**Зроблено, коли:**

- `plane` (або `plane_scale`) потрапляє в `RaycastScene` і в uniform шейдера.
- Літерал `0.66` у frag **видалено**.
- Зміна `plane_scale` у `player.toml` змінює видимий FOV без правки шейдера.

---

## P2 — Розділити `App`: host vs session

**Навіщо.** `App` — вікно, цикл, інпут, аудіо, гравець, UID демо. Наступна мапа чи сутність осідатиме ще полями в `engine::app`. Двигун зараз не хост, а сама гра. Див. [ARCHITECTURE.md](ARCHITECTURE.md) §2, §7.

**Де:**

- `src/engine/app.rs` — WASD / стрілки / Escape, `map::DEMO`, `texture::{BRICK,FLOOR,SKY}`, `sound_preset::{MUSIC,FOOTSTEPS}`, `upload_gpu_resources`, `update`/`render`.
- `src/engine/mod.rs` — реекспорт `run`.
- `src/game/` — зараз лише `Player`, `HeadBob`, `PlayerConfig`; туди (або в окремий demo-модуль) мають піти прив’язки контенту.
- Окремого `src/input/` **немає** — його не вигадувати як уже існуючий; з’явиться лише якщо цей пункт його додасть.

**Зроблено, коли:**

- Host (engine): event loop, `WindowContext`, таймінг, буфер інпуту, `request_redraw` / present.
- Session / demo: яка мапа, які текстури, які лупи, WASD → `Player::move_relative`.
- `engine::app` більше не імпортує конкретні `texture::BRICK` / `map::DEMO` як єдinu «гру».
- Поведінка демо (рух, bob, кроки, музика) зберігається; це рефактор, не нова гра.

---

## P3 — `load_all` → `Result`; resources не знають render; пресети за `SoundUid`

**Навіщо.** Лоад панікує на першому битому файлі; `TypedStore::get` панікує на відсутньому UID. `ConfigAsset::post_fx` тягне типи з `render` у шар ассетів. Пресет `clip = "footsteps"` резолвиться рядком (`sound_by_name`), не UID. Це ламає шар і ускладнює більше ассетів. Див. [ARCHITECTURE.md](ARCHITECTURE.md) §2, §5, §6, §9.

**Де:**

- `src/resources/manager.rs` — `ResourceManager::load_all()` зараз `-> Self` з `.expect` на кожен файл; `TypedStore::get` — `panic!("missing resource uid")`.
- `src/resources/types/config.rs` — `post_fx() -> Result<PostFxSettings, _>` імпортує `crate::render`.
- `src/engine/app.rs` — `resources.config(config::POSTPROCESS).post_fx().expect(...)`.
- `src/audio/presets.rs` — `resources.sound_by_name(&asset.clip)`.
- `src/resources/types/sound_preset.rs` — поле `clip: String`.
- `assets/configs/sounds/footsteps.toml`, `music.toml` — `clip = "footsteps"` / `"music"`.
- `src/resources/manager.rs` — `sound_by_name`.

**Зроблено, коли:**

- `ResourceManager::load_all() -> Result<Self, AssetError>` (або еквівалент); `App` / `run` обробляють помилку без прихованого panic у лоадері.
- Парсинг `PostFxSettings` живе в `render` або `game` з generic TOML, не всередині `ConfigAsset` як знання про vignette.
- Пресет посилається на `SoundUid` (або стабільний ID з маніфесту), не на рядок імені кліпу; `sound_by_name` не обов’язковий для цього шляху.

---

## P4 — Згорнути або доробити stub’и; `GlTexture` Drop

**Навіщо.** `MixerState` — порожні хуки. `RenderBackend` є, post — ні. `GlTexture` без `Drop`; `OpenGlBackend::drop` чистить program + VAO, не текстури; повторний upload витікає handle. Див. [ARCHITECTURE.md](ARCHITECTURE.md) §2, §4.

**Де:**

- `src/audio/mixer.rs` — `set_ducking` / `set_reverb_send` / `set_eq_low` порожні; `AudioEngine` тримає `MixerState`.
- `src/audio/volume.rs` — невикористані `linear_to_kira`, `combine_linear`.
- `src/audio/spatial.rs` — `let _ = linear_to_decibels(volume_linear)`.
- `src/render/api.rs` — трейт `RenderBackend`.
- `src/render/pipeline.rs` — хардкод `OpenGlBackend` + `OpenGlPostFx`.
- `src/render/postprocess/api.rs` — `PostProcessBackend` з `&glow::Context`.
- `src/render/backend/opengl/mod.rs` — `struct GlTexture(glow::NativeTexture)`; `Drop` для backend без delete текстур.
- `src/render/raycast.rs` — `set_*` замінює `Option<B::Texture>` без звільнення попередньої.

**Зроблено, коли (один з двох шляхів, свідомо):**

- **Згорнути:** видалити або не експортувати мертвий mixer/volume helpers; не прикидатися multi-backend, поки post прив’язаний до glow; **або**
- **Доробити:** mixer щось робить, post за тим самим шовом, що й raycast.
- І в будь-якому разі: `GlTexture` (або еквівалент) звільняє `NativeTexture` на drop; повторний `set_wall_texture` не лишає orphan GL-ім’я.

---

## P5 — Тести, `rust-version`, README, Git LFS

**Навіщо.** Єдиний чистий код (парсер мапи, колізія, bob, FOV-шлях після P1) не покритий. Немає `rust-version` / README — 1.83 навіть не парсить lockfile. PNG/OGG/MP3 у LFS; clone з `GIT_LFS_SKIP_SMUDGE` дає pointer-файли. Див. [ARCHITECTURE.md](ARCHITECTURE.md) §8–§9.

**Де:**

- `src/resources/types/map.rs` — `parse_row`, `MapAsset::load`, `is_wall`.
- `src/game/player.rs` — `move_relative`.
- `src/game/head_bob.rs` — `HeadBob::update`.
- Після P1 — шлях FOV (scene → uniform → шейдер) хоча б юніт-тестом збору `RaycastScene` / константи plane.
- `Cargo.toml` — немає `rust-version`.
- Кореня репо — не було README до цього PR; короткий README має лише вказувати на ці docs.
- `.gitattributes` — LFS для `*.png` `*.ogg` `*.mp3`.
- `src/resources/paths.rs` — `PSEUDO3D_ASSETS`.

**Зроблено, коли:**

- Є `#[test]` на парсинг `demo.map` (або фікстури), на `is_wall` / `move_relative` (не проходить крізь `#`), на оновлення bob (зміщення змінюється при `bob_speed > 0`).
- У `Cargo.toml` є `rust-version = "1.85"` (або фактичний мінімум lockfile).
- README (короткий) посилається на [ARCHITECTURE.md](ARCHITECTURE.md) і цей файл, збірку (Rust ≥ 1.85) і `git lfs pull`.
- У docs або README є примітка: без smudge LFS ассети — pointer-файли, потрібен `git lfs pull`.

---

## Порядок

P0 блокує візуальну перевірку. P1 — локальний, можна паралельно з P0. P2 легше після P0 (є живе вікно). P3 не залежить від GL. P4 — прибирання після того, як шар стабілізується. P5 можна починати з юнітів мапи/гравця одразу, `rust-version`/README — разом із будь-яким PR.

Деталі «як є»: [ARCHITECTURE.md](ARCHITECTURE.md).
