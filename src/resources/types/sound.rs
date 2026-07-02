use std::path::Path;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct SoundAsset {
    pub path: String,
}

impl Asset for SoundAsset {
    fn load(path: &Path) -> Result<Self, AssetError> {
        if !path.is_file() {
            return Err(AssetError::InvalidSound {
                path: path.display().to_string(),
                reason: "sound file not found".to_string(),
            });
        }

        Ok(Self {
            path: path.display().to_string(),
        })
    }
}
