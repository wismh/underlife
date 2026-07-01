use crate::resources::types::map::MapAsset;
use crate::resources::types::texture::TextureAsset;
use crate::render::api::{MapView, RaycastScene, RenderBackend};

pub struct RaycastRenderer<B: RenderBackend> {
    backend: B,
    wall: Option<B::Texture>,
    floor: Option<B::Texture>,
    ceiling: Option<B::Texture>,
    map: Option<B::Texture>,
    map_width: u32,
    map_height: u32,
}

impl<B: RenderBackend> RaycastRenderer<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            wall: None,
            floor: None,
            ceiling: None,
            map: None,
            map_width: 0,
            map_height: 0,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.backend.resize(width, height);
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn set_wall_texture(&mut self, texture: &TextureAsset) {
        self.wall = Some(self.backend.upload_texture_rgba(texture.as_view()));
    }

    pub fn set_floor_texture(&mut self, texture: &TextureAsset) {
        self.floor = Some(self.backend.upload_texture_rgba(texture.as_view()));
    }

    pub fn set_ceiling_texture(&mut self, texture: &TextureAsset) {
        self.ceiling = Some(self.backend.upload_texture_rgba(texture.as_view()));
    }

    pub fn set_map(&mut self, map: &MapAsset) {
        let cells = map.cells_r8();
        self.map = Some(
            self.backend
                .upload_texture_r8(&cells, map.width, map.height),
        );
        self.map_width = map.width;
        self.map_height = map.height;
    }

    pub fn draw(&mut self, scene: &RaycastScene) {
        let wall = self.wall.as_ref().expect("wall texture not uploaded");
        let floor = self.floor.as_ref().expect("floor texture not uploaded");
        let ceiling = self.ceiling.as_ref().expect("ceiling texture not uploaded");
        let map = self.map.as_ref().expect("map not uploaded");

        let map_view = MapView {
            width: self.map_width,
            height: self.map_height,
        };

        self.backend.begin_frame();
        self.backend
            .draw_raycast(scene, map, map_view, wall, floor, ceiling);
        self.backend.end_frame();
    }
}
