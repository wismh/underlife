use std::path::Path;

use toml::Value;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct ConfigAsset {
    path: String,
    root: Value,
}

impl ConfigAsset {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn section(&self, key: &str) -> Result<&toml::map::Map<String, Value>, AssetError> {
        self.root
            .get(key)
            .and_then(|value| value.as_table())
            .ok_or_else(|| AssetError::InvalidConfig {
                path: self.path.clone(),
                reason: format!("missing [{key}] section"),
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
