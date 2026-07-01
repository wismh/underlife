use crate::render::api::RaycastScene;
use crate::render::backend::opengl::{OpenGlBackend, OpenGlPostFx};
use crate::render::postprocess::{PostFxSettings, PostProcessBackend};
use crate::render::raycast::RaycastRenderer;
use crate::resources::types::map::MapAsset;
use crate::resources::types::shader::ShaderAsset;
use crate::resources::types::texture::TextureAsset;

pub struct RenderPipeline {
    raycast: RaycastRenderer<OpenGlBackend>,
    post: OpenGlPostFx,
}

impl RenderPipeline {
    pub fn new(
        gl: glow::Context,
        raycast_shader: &ShaderAsset,
        post_shader: &ShaderAsset,
    ) -> Self {
        let backend = OpenGlBackend::new(gl, raycast_shader);
        let post = OpenGlPostFx::new(backend.context(), post_shader);
        Self {
            raycast: RaycastRenderer::new(backend),
            post,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.raycast.resize(width, height);
        let gl = self.raycast.backend().context();
        self.post.resize_postprocess(gl, width, height);
    }

    pub fn set_wall_texture(&mut self, texture: &TextureAsset) {
        self.raycast.set_wall_texture(texture);
    }

    pub fn set_floor_texture(&mut self, texture: &TextureAsset) {
        self.raycast.set_floor_texture(texture);
    }

    pub fn set_ceiling_texture(&mut self, texture: &TextureAsset) {
        self.raycast.set_ceiling_texture(texture);
    }

    pub fn set_map(&mut self, map: &MapAsset) {
        self.raycast.set_map(map);
    }

    pub fn draw(&mut self, scene: &RaycastScene, post_fx: &PostFxSettings) {
        self.post
            .begin_scene_pass(self.raycast.backend().context());
        self.raycast.draw(scene);
        self.post.apply_postprocess(
            self.raycast.backend().context(),
            post_fx,
            scene.width,
            scene.height,
        );
    }
}

impl Drop for RenderPipeline {
    fn drop(&mut self) {
        let gl = self.raycast.backend().context();
        self.post.destroy(gl);
    }
}
