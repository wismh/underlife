use std::path::Path;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Asset for TextureAsset {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let image = image::open(path).map_err(|source| AssetError::Decode {
            path: path.display().to_string(),
            source,
        })?;
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        Ok(Self {
            width,
            height,
            rgba: image.into_raw(),
        })
    }
}

impl TextureAsset {
    pub fn as_view(&self) -> TextureView<'_> {
        TextureView {
            width: self.width,
            height: self.height,
            rgba: &self.rgba,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureView<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}
