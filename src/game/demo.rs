use std::collections::HashSet;
use std::time::Duration;

use winit::keyboard::KeyCode;

use crate::audio::{AudioEngine, LoopHandle, PlayParams, SoundSource};
use crate::game::{HeadBob, Player, PlayerConfig};
use crate::render::{PostFxSettings, RaycastScene, RenderPipeline};
use crate::resources::assets::{config, map, shader, sound_preset, texture};
use crate::resources::manager::ResourceManager;
use crate::resources::types::shader::ShaderAsset;
use crate::resources::AssetError;

/// Demo walkthrough: which map/textures/loops play, and how WASD moves the player.
pub struct DemoSession {
    resources: ResourceManager,
    audio: Option<AudioEngine>,
    player: Player,
    head_bob: HeadBob,
    post_fx: PostFxSettings,
    footstep_loop: Option<LoopHandle>,
    player_cfg: PlayerConfig,
    mouse_look: bool,
}

impl DemoSession {
    pub fn new() -> Result<Self, AssetError> {
        let resources = ResourceManager::load_all()?;
        let post_fx = PostFxSettings::from_config(resources.config(config::POSTPROCESS))?;
        let player_cfg = PlayerConfig::load(resources.config(config::PLAYER))?;
        let mut audio = match AudioEngine::new(&resources) {
            Ok(engine) => Some(engine),
            Err(error) => {
                eprintln!("[audio] init failed ({error}); starting without music/SFX");
                None
            }
        };
        let footstep_loop = audio.as_mut().map(|engine| engine.create_loop_handle());

        Ok(Self {
            player: Player::from_spawn(player_cfg.spawn, player_cfg.movement),
            head_bob: HeadBob::new(),
            mouse_look: false,
            post_fx,
            footstep_loop,
            player_cfg,
            audio,
            resources,
        })
    }

    pub fn raycast_shader(&self) -> &ShaderAsset {
        self.resources.shader(shader::RAYCAST)
    }

    pub fn post_shader(&self) -> &ShaderAsset {
        self.resources.shader(shader::POSTPROCESS)
    }

    pub fn bind_renderer(&self, renderer: &mut RenderPipeline) {
        renderer.set_wall_texture(self.resources.texture(texture::BRICK));
        renderer.set_floor_texture(self.resources.texture(texture::FLOOR));
        renderer.set_ceiling_texture(self.resources.texture(texture::SKY));
        renderer.set_map(self.resources.map(map::DEMO));
    }

    pub fn start_music(&mut self) {
        if let Some(audio) = self.audio.as_mut() {
            audio.play_music(
                SoundSource::Preset(sound_preset::MUSIC),
                true,
                Some(Duration::from_secs(2)),
            );
        }
    }

    pub fn set_mouse_look(&mut self, enabled: bool) {
        self.mouse_look = enabled;
    }

    pub fn on_mouse_motion(&mut self, delta_x: f64) {
        if !self.mouse_look {
            return;
        }
        self.player
            .rotate((delta_x as f32) * self.player_cfg.movement.mouse_sensitivity);
    }

    pub fn update(&mut self, dt: f32, keys: &HashSet<KeyCode>) {
        let movement = self.player_cfg.movement;
        let head_bob = self.player_cfg.head_bob;

        if !self.mouse_look {
            let rotate = (key_down(keys, KeyCode::ArrowLeft) as i32
                - key_down(keys, KeyCode::ArrowRight) as i32) as f32;
            if rotate != 0.0 {
                self.player.rotate(-rotate * movement.rotate_speed * dt);
            }
        }

        let forward =
            (key_down(keys, KeyCode::KeyW) as i32 - key_down(keys, KeyCode::KeyS) as i32) as f32;
        let strafe =
            (key_down(keys, KeyCode::KeyD) as i32 - key_down(keys, KeyCode::KeyA) as i32) as f32;

        let bob_speed =
            glam::Vec2::new(forward * movement.move_speed, strafe * movement.move_speed).length();
        self.head_bob.update(dt, bob_speed, head_bob, movement);

        if forward != 0.0 || strafe != 0.0 {
            self.player.move_relative(
                self.resources.map(map::DEMO),
                forward,
                strafe,
                movement.move_speed * dt,
                movement.strafe_speed * dt,
            );
        }

        let moving = forward != 0.0 || strafe != 0.0;
        if let (Some(audio), Some(handle)) = (self.audio.as_mut(), self.footstep_loop) {
            audio.update_loop(
                handle,
                moving,
                SoundSource::Preset(sound_preset::FOOTSTEPS),
                PlayParams::default(),
            );
        }

        if let Some(audio) = self.audio.as_mut() {
            audio.set_listener(
                glam::Vec3::new(self.player.pos.x, self.player.pos.y, 0.0),
                glam::Vec3::new(self.player.dir.x, self.player.dir.y, 0.0),
            );
            audio.update(dt);
        }
    }

    pub fn draw(&self, renderer: &mut RenderPipeline, width: u32, height: u32) {
        let scene = assemble_scene(width, height, &self.player, &self.head_bob);
        renderer.draw(&scene, &self.post_fx);
    }
}

pub(crate) fn assemble_scene(
    width: u32,
    height: u32,
    player: &Player,
    head_bob: &HeadBob,
) -> RaycastScene {
    RaycastScene {
        width,
        height,
        player_pos: [player.pos.x, player.pos.y],
        player_dir: [player.dir.x, player.dir.y],
        player_plane: [player.plane.x, player.plane.y],
        view_bob: [head_bob.offset_x, head_bob.offset_y],
    }
}

fn key_down(keys: &HashSet<KeyCode>, key: KeyCode) -> bool {
    keys.contains(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_uses_player_plane_from_plane_scale() {
        let player = Player::new(2.5, 2.5, 0.0, 0.66);
        let bob = HeadBob::new();
        let scene = assemble_scene(320, 200, &player, &bob);

        assert_eq!(scene.player_pos, [2.5, 2.5]);
        assert!((scene.player_dir[0] - 1.0).abs() < 1e-5);
        assert!(scene.player_dir[1].abs() < 1e-5);
        assert_eq!(scene.player_plane[0], player.plane.x);
        assert_eq!(scene.player_plane[1], player.plane.y);
        // yaw 0 → dir (1, 0), plane = perpendicular(dir) * 0.66 = (0, 0.66)
        assert!(scene.player_plane[0].abs() < 1e-5);
        assert!((scene.player_plane[1] - 0.66).abs() < 1e-5);

        let narrow = Player::new(2.5, 2.5, 0.0, 0.40);
        let narrow_scene = assemble_scene(320, 200, &narrow, &bob);
        assert!((narrow_scene.player_plane[1] - 0.40).abs() < 1e-5);
        assert!((narrow_scene.player_plane[1] - scene.player_plane[1]).abs() > 0.1);
    }

    #[test]
    fn raycast_shader_reads_plane_uniform_not_hardcoded_fov() {
        let src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/raycast.frag"
        ));
        assert!(src.contains("u_player_plane"));
        assert!(
            !src.contains("* 0.66"),
            "FOV scale must come from the camera uniform, not a shader literal"
        );
    }
}
