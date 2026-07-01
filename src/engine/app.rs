use std::collections::HashSet;
use std::time::Instant;

use crate::render::{PostFxSettings, RaycastScene};
use crate::resources::manager::ResourceManager;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, DeviceEvents, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowId};

use crate::engine::window::{EngineConfig, WindowContext};
use crate::game::{default_player, HeadBob, Player, MOVE_SPEED, MOUSE_SENSITIVITY, ROTATE_SPEED, STRAFE_SPEED};
use crate::resources::assets::{config, map, shader, texture};

pub fn run() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.listen_device_events(DeviceEvents::WhenFocused);
    let mut app = App::new();
    event_loop.run_app(&mut app)
}

struct App {
    window: Option<WindowContext>,
    resources: ResourceManager,
    player: Player,
    head_bob: HeadBob,
    keys: HashSet<KeyCode>,
    last_frame: Instant,
    mouse_look: bool,
    post_fx: PostFxSettings,
}

impl App {
    fn new() -> Self {
        let resources = ResourceManager::load_all();
        let post_fx = resources
            .config(config::POSTPROCESS)
            .post_fx()
            .expect("parse postprocess config");

        Self {
            window: None,
            resources,
            player: default_player(),
            head_bob: HeadBob::new(),
            keys: HashSet::new(),
            last_frame: Instant::now(),
            mouse_look: false,
            post_fx,
        }
    }

    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let raycast_shader = self.resources.shader(shader::RAYCAST);
        let post_shader = self.resources.shader(shader::POSTPROCESS);
        let mut window = WindowContext::create(
            event_loop,
            &EngineConfig::default(),
            raycast_shader,
            post_shader,
        );
        self.upload_gpu_resources(&mut window);
        Self::capture_mouse(&window);
        self.mouse_look = true;
        self.window = Some(window);
    }

    fn capture_mouse(window: &WindowContext) {
        window
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .expect("capture mouse");
        window.window.set_cursor_visible(false);
    }

    fn release_mouse(window: &WindowContext) {
        let _ = window.window.set_cursor_grab(CursorGrabMode::None);
        window.window.set_cursor_visible(true);
    }

    fn upload_gpu_resources(&mut self, window: &mut WindowContext) {
        let wall = self.resources.texture(texture::BRICK);
        let floor = self.resources.texture(texture::FLOOR);
        let ceiling = self.resources.texture(texture::SKY);
        let level = self.resources.map(map::DEMO);

        window.renderer.set_wall_texture(wall);
        window.renderer.set_floor_texture(floor);
        window.renderer.set_ceiling_texture(ceiling);
        window.renderer.set_map(level);
    }

    fn update(&mut self, dt: f32) {
        if !self.mouse_look {
            let rotate = (self.key_down(KeyCode::ArrowLeft) as i32
                - self.key_down(KeyCode::ArrowRight) as i32) as f32;
            if rotate != 0.0 {
                self.player.rotate(-rotate * ROTATE_SPEED * dt);
            }
        }

        let forward = (self.key_down(KeyCode::KeyW) as i32 - self.key_down(KeyCode::KeyS) as i32)
            as f32;
        let strafe =
            (self.key_down(KeyCode::KeyD) as i32 - self.key_down(KeyCode::KeyA) as i32) as f32;

        let bob_speed = glam::Vec2::new(forward * MOVE_SPEED, strafe * MOVE_SPEED).length();
        self.head_bob.update(dt, bob_speed);

        if forward != 0.0 || strafe != 0.0 {
            self.player.move_relative(
                self.resources.map(map::DEMO),
                forward,
                strafe,
                MOVE_SPEED * dt,
                STRAFE_SPEED * dt,
            );
        }
    }

    fn render(&mut self) {
        let Some(window) = self.window.as_mut() else {
            return;
        };

        let (width, height) = window.inner_size();
        let scene = RaycastScene {
            width,
            height,
            player_pos: [self.player.pos.x, self.player.pos.y],
            player_dir: [self.player.dir.x, self.player.dir.y],
            view_bob: [self.head_bob.offset_x, self.head_bob.offset_y],
        };

        window.renderer.draw(&scene, &self.post_fx);
        window.present();
    }

    fn key_down(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_window(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(focused) => {
                if let Some(window) = self.window.as_ref() {
                    if focused {
                        Self::capture_mouse(window);
                        self.mouse_look = true;
                    } else {
                        Self::release_mouse(window);
                        self.mouse_look = false;
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(window) = self.window.as_mut() {
                    window.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state,
                    ..
                } => match state {
                    ElementState::Pressed => {
                        if code == KeyCode::Escape {
                            event_loop.exit();
                        }
                        self.keys.insert(code);
                    }
                    ElementState::Released => {
                        self.keys.remove(&code);
                    }
                },
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.mouse_look {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            self.player.rotate((delta.0 as f32) * MOUSE_SENSITIVITY);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;

        self.update(dt);

        if let Some(window) = &self.window {
            window.window.request_redraw();
        }
    }
}
