pub use crate::resources::types::texture::TextureView;

#[derive(Debug, Clone, Copy)]
pub struct RaycastScene {
    pub width: u32,
    pub height: u32,
    pub player_pos: [f32; 2],
    pub player_dir: [f32; 2],
    /// x: horizontal sway in world units, y: vertical bob in screen pixels
    pub view_bob: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct MapView {
    pub width: u32,
    pub height: u32,
}

pub trait RenderBackend {
    type Texture;

    fn resize(&mut self, width: i32, height: i32);
    fn begin_frame(&mut self);
    fn upload_texture_rgba(&mut self, view: TextureView<'_>) -> Self::Texture;
    fn upload_texture_r8(&mut self, cells: &[u8], width: u32, height: u32) -> Self::Texture;
    fn draw_raycast(
        &mut self,
        scene: &RaycastScene,
        map: &Self::Texture,
        map_size: MapView,
        wall: &Self::Texture,
        floor: &Self::Texture,
        ceiling: &Self::Texture,
    );
    fn end_frame(&mut self);
}
