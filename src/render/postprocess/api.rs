#[derive(Debug, Clone, Copy)]
pub struct PostFxSettings {
    pub vignette: VignetteSettings,
}

impl Default for PostFxSettings {
    fn default() -> Self {
        Self {
            vignette: VignetteSettings::default(),
        }
    }
}

// Matches Unity Post Processing Stack "Classic" vignette parameters.
#[derive(Debug, Clone, Copy)]
pub struct VignetteSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub smoothness: f32,
    pub roundness: f32,
    pub rounded: bool,
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.2,
            smoothness: 0.2,
            roundness: 1.0,
            rounded: true,
        }
    }
}

impl VignetteSettings {
    pub fn unity_shader_settings(&self) -> (f32, f32, f32, f32) {
        let roundness = (1.0 - self.roundness) * 6.0 + self.roundness;
        (
            self.intensity * 3.0,
            self.smoothness * 5.0,
            roundness,
            if self.rounded { 1.0 } else { 0.0 },
        )
    }
}

pub trait PostProcessBackend {
    fn resize_postprocess(&mut self, gl: &glow::Context, width: i32, height: i32);
    fn begin_scene_pass(&self, gl: &glow::Context);
    fn apply_postprocess(
        &self,
        gl: &glow::Context,
        settings: &PostFxSettings,
        width: u32,
        height: u32,
    );
}
