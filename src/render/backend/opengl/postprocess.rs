use glow::HasContext;

use super::compile_program;
use crate::render::postprocess::{PostFxSettings, PostProcessBackend};
use crate::resources::types::shader::ShaderAsset;

pub struct OpenGlPostFx {
    program: glow::NativeProgram,
    vao: glow::NativeVertexArray,
    scene_fbo: glow::NativeFramebuffer,
    scene_color: glow::NativeTexture,
    width: u32,
    height: u32,
    u_resolution: glow::NativeUniformLocation,
    u_vignette_enabled: glow::NativeUniformLocation,
    u_vignette_intensity: glow::NativeUniformLocation,
    u_vignette_smoothness: glow::NativeUniformLocation,
    u_vignette_roundness: glow::NativeUniformLocation,
    u_vignette_rounded: glow::NativeUniformLocation,
}

impl OpenGlPostFx {
    pub fn new(gl: &glow::Context, shader: &ShaderAsset) -> Self {
        unsafe {
            let program = compile_program(gl, &shader.vertex, &shader.fragment);
            let vao = gl.create_vertex_array().expect("create post VAO");
            gl.bind_vertex_array(Some(vao));

            let u_resolution = gl.get_uniform_location(program, "u_resolution").unwrap();
            let u_scene = gl.get_uniform_location(program, "u_scene").unwrap();
            let u_vignette_enabled =
                gl.get_uniform_location(program, "u_vignette_enabled").unwrap();
            let u_vignette_intensity =
                gl.get_uniform_location(program, "u_vignette_intensity").unwrap();
            let u_vignette_smoothness =
                gl.get_uniform_location(program, "u_vignette_smoothness").unwrap();
            let u_vignette_roundness =
                gl.get_uniform_location(program, "u_vignette_roundness").unwrap();
            let u_vignette_rounded =
                gl.get_uniform_location(program, "u_vignette_rounded").unwrap();

            gl.use_program(Some(program));
            gl.uniform_1_i32(Some(&u_scene), 0);

            let (scene_fbo, scene_color) = create_scene_target(gl, 1, 1);

            Self {
                program,
                vao,
                scene_fbo,
                scene_color,
                width: 1,
                height: 1,
                u_resolution,
                u_vignette_enabled,
                u_vignette_intensity,
                u_vignette_smoothness,
                u_vignette_roundness,
                u_vignette_rounded,
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_framebuffer(self.scene_fbo);
            gl.delete_texture(self.scene_color);
        }
    }

    fn recreate_scene_target(&mut self, gl: &glow::Context, width: u32, height: u32) {
        unsafe {
            gl.delete_framebuffer(self.scene_fbo);
            gl.delete_texture(self.scene_color);
            let (fbo, color) = create_scene_target(gl, width, height);
            self.scene_fbo = fbo;
            self.scene_color = color;
            self.width = width;
            self.height = height;
        }
    }
}

impl PostProcessBackend for OpenGlPostFx {
    fn resize_postprocess(&mut self, gl: &glow::Context, width: i32, height: i32) {
        let width = width.max(1) as u32;
        let height = height.max(1) as u32;
        if width != self.width || height != self.height {
            self.recreate_scene_target(gl, width, height);
        }
    }

    fn begin_scene_pass(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.scene_fbo));
            gl.viewport(0, 0, self.width as i32, self.height as i32);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    fn apply_postprocess(
        &self,
        gl: &glow::Context,
        settings: &PostFxSettings,
        width: u32,
        height: u32,
    ) {
        let vignette = settings.vignette;
        let (intensity, smoothness, roundness, rounded) = vignette.unity_shader_settings();
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear(glow::COLOR_BUFFER_BIT);

            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.scene_color));

            gl.uniform_2_f32(Some(&self.u_resolution), width as f32, height as f32);
            gl.uniform_1_i32(
                Some(&self.u_vignette_enabled),
                vignette.enabled as i32,
            );
            gl.uniform_1_f32(Some(&self.u_vignette_intensity), intensity);
            gl.uniform_1_f32(Some(&self.u_vignette_smoothness), smoothness);
            gl.uniform_1_f32(Some(&self.u_vignette_roundness), roundness);
            gl.uniform_1_f32(Some(&self.u_vignette_rounded), rounded);

            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

unsafe fn create_scene_target(
    gl: &glow::Context,
    width: u32,
    height: u32,
) -> (glow::NativeFramebuffer, glow::NativeTexture) {
    let color = gl.create_texture().expect("create scene color texture");
    gl.bind_texture(glow::TEXTURE_2D, Some(color));
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_S,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_T,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA8 as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        glow::PixelUnpackData::Slice(None),
    );

    let fbo = gl.create_framebuffer().expect("create scene framebuffer");
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(color),
        0,
    );

    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if status != glow::FRAMEBUFFER_COMPLETE {
        panic!("scene framebuffer incomplete: {status}");
    }

    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    (fbo, color)
}
