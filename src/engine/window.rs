use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, ContextApi, Version};
use glutin::display::GetGlDisplay;
use glutin::display::GlDisplay;
use glutin::prelude::*;
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use raw_window_handle::HasWindowHandle;
use crate::render::backend::opengl::{OpenGlBackend, pick_gl_config};
use crate::render::RaycastRenderer;
use crate::resources::types::shader::ShaderAsset;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "pseudo3d".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

pub struct WindowContext {
    pub window: Arc<Window>,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    pub renderer: RaycastRenderer<OpenGlBackend>,
}

impl WindowContext {
    pub fn create(
        event_loop: &ActiveEventLoop,
        config: &EngineConfig,
        shader: &ShaderAsset,
    ) -> Self {
        let window_attributes = Window::default_attributes()
            .with_title(config.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                config.width as f64,
                config.height as f64,
            ));

        let (window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(
                event_loop,
                ConfigTemplateBuilder::new(),
                pick_gl_config,
            )
            .expect("create window and OpenGL config");

        let window = Arc::new(window.expect("platform did not create a window"));
        let gl_display = gl_config.display();

        let raw_window_handle = window.window_handle().expect("window handle").as_raw();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window_handle));

        let not_current = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("create GL context")
        };

        let (width, height) = inner_size(&window);
        let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(
                raw_window_handle,
                NonZeroU32::new(width.max(1) as u32).unwrap(),
                NonZeroU32::new(height.max(1) as u32).unwrap(),
            );

        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .expect("create GL surface")
        };

        let context = not_current
            .make_current(&surface)
            .expect("make GL context current");

        surface
            .set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .expect("set swap interval");

        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                let name = CString::new(name).expect("proc name");
                gl_display.get_proc_address(&name)
            })
        };

        let backend = OpenGlBackend::new(gl, shader);
        let mut renderer = RaycastRenderer::new(backend);
        renderer.resize(width, height);

        Self {
            window,
            surface,
            context,
            renderer,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        if width <= 0 || height <= 0 {
            return;
        }

        self.renderer.resize(width, height);
        self.surface.resize(
            &self.context,
            NonZeroU32::new(width as u32).unwrap(),
            NonZeroU32::new(height as u32).unwrap(),
        );
    }

    pub fn present(&mut self) {
        self.surface
            .swap_buffers(&self.context)
            .expect("swap buffers");
    }

    pub fn inner_size(&self) -> (u32, u32) {
        let (width, height) = inner_size(&self.window);
        (width.max(1) as u32, height.max(1) as u32)
    }
}

fn inner_size(window: &Window) -> (i32, i32) {
    let size = window.inner_size();
    (size.width as i32, size.height as i32)
}
