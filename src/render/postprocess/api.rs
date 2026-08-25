use crate::resources::types::config::ConfigAsset;
use crate::resources::AssetError;

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

impl PostFxSettings {
    pub fn from_config(config: &ConfigAsset) -> Result<Self, AssetError> {
        let table = config.section("vignette")?;
        Ok(Self {
            vignette: VignetteSettings {
                enabled: get_bool(table, "enabled", true),
                intensity: get_f32(config.path(), table, "intensity")?,
                smoothness: get_f32(config.path(), table, "smoothness")?,
                roundness: get_f32(config.path(), table, "roundness")?,
                rounded: get_bool(table, "rounded", false),
            },
        })
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

fn get_bool(table: &toml::map::Map<String, toml::Value>, key: &str, default: bool) -> bool {
    table
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn get_f32(
    path: &str,
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<f32, AssetError> {
    let Some(value) = table.get(key) else {
        return Err(AssetError::InvalidConfig {
            path: path.to_string(),
            reason: format!("missing key `{key}`"),
        });
    };

    value
        .as_float()
        .map(|number| number as f32)
        .or_else(|| value.as_integer().map(|number| number as f32))
        .ok_or_else(|| AssetError::InvalidConfig {
            path: path.to_string(),
            reason: format!("`{key}` must be a number"),
        })
}
