use glow::HasContext;
use glutin::config::GlConfig;

use crate::render::api::{MapView, RaycastScene, RenderBackend, TextureView};
use crate::resources::types::shader::ShaderAsset;

pub struct GlTexture(glow::NativeTexture);

pub struct OpenGlBackend {
    gl: glow::Context,
    program: glow::NativeProgram,
    vao: glow::NativeVertexArray,
    u_resolution: glow::NativeUniformLocation,
    u_player_pos: glow::NativeUniformLocation,
    u_player_dir: glow::NativeUniformLocation,
    u_map_size: glow::NativeUniformLocation,
}

impl OpenGlBackend {
    pub fn new(gl: glow::Context, shader: &ShaderAsset) -> Self {
        unsafe {
            let program = compile_program(&gl, &shader.vertex, &shader.fragment);

            let vao = gl.create_vertex_array().expect("create VAO");
            gl.bind_vertex_array(Some(vao));

            let u_resolution = gl.get_uniform_location(program, "u_resolution").unwrap();
            let u_player_pos = gl.get_uniform_location(program, "u_player_pos").unwrap();
            let u_player_dir = gl.get_uniform_location(program, "u_player_dir").unwrap();
            let u_map_size = gl.get_uniform_location(program, "u_map_size").unwrap();

            let u_map = gl.get_uniform_location(program, "u_map").unwrap();
            let u_wall_tex = gl.get_uniform_location(program, "u_wall_tex").unwrap();
            let u_floor_tex = gl.get_uniform_location(program, "u_floor_tex").unwrap();
            let u_ceiling_tex = gl.get_uniform_location(program, "u_ceiling_tex").unwrap();

            gl.use_program(Some(program));
            gl.uniform_1_i32(Some(&u_map), 0);
            gl.uniform_1_i32(Some(&u_wall_tex), 1);
            gl.uniform_1_i32(Some(&u_floor_tex), 2);
            gl.uniform_1_i32(Some(&u_ceiling_tex), 3);

            gl.clear_color(0.05, 0.05, 0.08, 1.0);

            Self {
                gl,
                program,
                vao,
                u_resolution,
                u_player_pos,
                u_player_dir,
                u_map_size,
            }
        }
    }
}

impl RenderBackend for OpenGlBackend {
    type Texture = GlTexture;

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.viewport(0, 0, width, height);
        }
    }

    fn begin_frame(&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.use_program(Some(self.program));
            self.gl.bind_vertex_array(Some(self.vao));
        }
    }

    fn upload_texture_rgba(&mut self, view: TextureView<'_>) -> Self::Texture {
        unsafe {
            let texture = self.gl.create_texture().expect("create texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                view.width as i32,
                view.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(view.rgba)),
            );
            self.gl.generate_mipmap(glow::TEXTURE_2D);
            GlTexture(texture)
        }
    }

    fn upload_texture_r8(&mut self, cells: &[u8], width: u32, height: u32) -> Self::Texture {
        unsafe {
            let texture = self.gl.create_texture().expect("create map texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(cells)),
            );
            GlTexture(texture)
        }
    }

    fn draw_raycast(
        &mut self,
        scene: &RaycastScene,
        map: &Self::Texture,
        map_size: MapView,
        wall: &Self::Texture,
        floor: &Self::Texture,
        ceiling: &Self::Texture,
    ) {
        unsafe {
            self.gl.uniform_2_f32(
                Some(&self.u_resolution),
                scene.width as f32,
                scene.height as f32,
            );
            self.gl.uniform_2_f32(
                Some(&self.u_player_pos),
                scene.player_pos[0],
                scene.player_pos[1],
            );
            self.gl.uniform_2_f32(
                Some(&self.u_player_dir),
                scene.player_dir[0],
                scene.player_dir[1],
            );
            self.gl.uniform_2_f32(
                Some(&self.u_map_size),
                map_size.width as f32,
                map_size.height as f32,
            );

            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(map.0));
            self.gl.active_texture(glow::TEXTURE1);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(wall.0));
            self.gl.active_texture(glow::TEXTURE2);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(floor.0));
            self.gl.active_texture(glow::TEXTURE3);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(ceiling.0));

            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    fn end_frame(&mut self) {}
}

impl Drop for OpenGlBackend {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

pub fn pick_gl_config(
    configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>,
) -> glutin::config::Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() < accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .expect("no suitable OpenGL config found")
}

unsafe fn compile_program(
    gl: &glow::Context,
    vertex_src: &str,
    fragment_src: &str,
) -> glow::NativeProgram {
    let vertex = compile_shader(gl, glow::VERTEX_SHADER, vertex_src);
    let fragment = compile_shader(gl, glow::FRAGMENT_SHADER, fragment_src);

    let program = gl.create_program().expect("create program");
    gl.attach_shader(program, vertex);
    gl.attach_shader(program, fragment);
    gl.link_program(program);

    if !gl.get_program_link_status(program) {
        panic!(
            "Program link error: {}",
            gl.get_program_info_log(program)
        );
    }

    gl.delete_shader(vertex);
    gl.delete_shader(fragment);
    program
}

unsafe fn compile_shader(gl: &glow::Context, kind: u32, source: &str) -> glow::NativeShader {
    let shader = gl.create_shader(kind).expect("create shader");
    gl.shader_source(shader, source);
    gl.compile_shader(shader);

    if !gl.get_shader_compile_status(shader) {
        panic!(
            "Shader compile error ({}): {}",
            kind,
            gl.get_shader_info_log(shader)
        );
    }

    shader
}
