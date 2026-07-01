use std::path::Path;

use toml::Value;

use crate::render::{PostFxSettings, VignetteSettings};
use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct ConfigAsset {
    path: String,
    root: Value,
}

impl ConfigAsset {
    pub fn post_fx(&self) -> Result<PostFxSettings, AssetError> {
        let table = self.section("vignette")?;
        Ok(PostFxSettings {
            vignette: VignetteSettings {
                enabled: get_bool(table, "enabled", true),
                intensity: get_f32(&self.path, table, "intensity")?,
                smoothness: get_f32(&self.path, table, "smoothness")?,
                roundness: get_f32(&self.path, table, "roundness")?,
                rounded: get_bool(table, "rounded", false),
            },
        })
    }

    fn section(&self, key: &str) -> Result<&toml::map::Map<String, Value>, AssetError> {
        self.root.get(key).and_then(|value| value.as_table()).ok_or_else(|| {
            AssetError::InvalidConfig {
                path: self.path.clone(),
                reason: format!("missing [{key}] section"),
            }
        })
    }
}

impl Asset for ConfigAsset {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let text = std::fs::read_to_string(path).map_err(|source| AssetError::Io {
            path: path.display().to_string(),
            source,
        })?;

        let root: Value = toml::from_str(&text).map_err(|source| AssetError::InvalidConfig {
            path: path.display().to_string(),
            reason: source.to_string(),
        })?;

        if !root.is_table() {
            return Err(AssetError::InvalidConfig {
                path: path.display().to_string(),
                reason: "config root must be a TOML table".to_string(),
            });
        }

        Ok(Self {
            path: path.display().to_string(),
            root,
        })
    }
}

fn get_bool(table: &toml::map::Map<String, Value>, key: &str, default: bool) -> bool {
    table
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn get_f32(
    path: &str,
    table: &toml::map::Map<String, Value>,
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
