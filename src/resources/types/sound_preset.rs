use std::path::Path;

use toml::Value;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct SoundPresetAsset {
    pub clip: String,
    pub volume: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
}

impl Asset for SoundPresetAsset {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let text = std::fs::read_to_string(path).map_err(|source| AssetError::Io {
            path: path.display().to_string(),
            source,
        })?;

        let root: Value = toml::from_str(&text).map_err(|source| AssetError::InvalidConfig {
            path: path.display().to_string(),
            reason: source.to_string(),
        })?;

        let table = root.as_table().ok_or_else(|| AssetError::InvalidConfig {
            path: path.display().to_string(),
            reason: "sound preset root must be a TOML table".to_string(),
        })?;

        let path_str = path.display().to_string();
        Ok(Self {
            clip: get_string(table, "clip", &path_str)?,
            volume: get_f32(table, "volume", &path_str)?,
            pitch_min: get_f32(table, "pitch_min", &path_str)?,
            pitch_max: get_f32(table, "pitch_max", &path_str)?,
        })
    }
}

fn get_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<String, AssetError> {
    table
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .ok_or_else(|| AssetError::InvalidConfig {
            path: path.to_string(),
            reason: format!("missing or invalid string key `{key}`"),
        })
}

fn get_f32(
    table: &toml::map::Map<String, Value>,
    key: &str,
    path: &str,
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
