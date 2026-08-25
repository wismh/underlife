use std::collections::HashSet;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, DeviceEvents, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::engine::window::{EngineConfig, WindowContext};
use crate::engine::EngineError;
use crate::game::DemoSession;

/// Host: event loop, window, timing, input buffer, present.
/// Gameplay (map, textures, movement, audio) lives on [`DemoSession`].
pub fn run() -> Result<(), EngineError> {
    let event_loop = EventLoop::new()?;
    event_loop.listen_device_events(DeviceEvents::WhenFocused);
    let mut app = App::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    window: Option<WindowContext>,
    keys: HashSet<KeyCode>,
    last_frame: Instant,
    session: DemoSession,
}

impl App {
    fn new() -> Result<Self, EngineError> {
        Ok(Self {
            window: None,
            keys: HashSet::new(),
            last_frame: Instant::now(),
            session: DemoSession::new()?,
        })
    }

    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut window = WindowContext::create(
            event_loop,
            &EngineConfig::default(),
            self.session.raycast_shader(),
            self.session.post_shader(),
        );
        self.session.bind_renderer(&mut window.renderer);
        window.capture_mouse();
        self.session.set_mouse_look(true);
        self.session.start_music();
        self.window = Some(window);
    }

    fn render(&mut self) {
        let Some(window) = self.window.as_mut() else {
            return;
        };

        let (width, height) = window.inner_size();
        self.session.draw(&mut window.renderer, width, height);
        window.present();
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
                        window.capture_mouse();
                        self.session.set_mouse_look(true);
                    } else {
                        window.release_mouse();
                        self.session.set_mouse_look(false);
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
        if let DeviceEvent::MouseMotion { delta } = event {
            self.session.on_mouse_motion(delta.0);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;

        self.session.update(dt, &self.keys);

        if let Some(window) = &self.window {
            window.window.request_redraw();
        }
    }
}
