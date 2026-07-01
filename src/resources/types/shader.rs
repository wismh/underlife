use std::path::Path;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct ShaderAsset {
    pub vertex: String,
    pub fragment: String,
}

impl ShaderAsset {
    pub fn load_pair(vertex_path: &Path, fragment_path: &Path) -> Result<Self, AssetError> {
        let vertex = std::fs::read_to_string(vertex_path).map_err(|source| AssetError::Io {
            path: vertex_path.display().to_string(),
            source,
        })?;
        let fragment = std::fs::read_to_string(fragment_path).map_err(|source| AssetError::Io {
            path: fragment_path.display().to_string(),
            source,
        })?;

        Ok(Self { vertex, fragment })
    }
}

impl Asset for ShaderAsset {
    fn load(_path: &Path) -> Result<Self, AssetError> {
        Err(AssetError::UnsupportedLoader {
            kind: "shader",
            hint: "use ShaderAsset::load_pair(vert, frag) or manifest [[shader]] entries",
        })
    }
}
