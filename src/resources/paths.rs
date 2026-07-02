use std::env;
use std::path::PathBuf;

use crate::resources::asset::AssetError;

const ASSETS_DIR_NAME: &str = "assets";
const ASSETS_OVERRIDE_ENV: &str = "PSEUDO3D_ASSETS";

pub fn resolve_assets_root() -> Result<PathBuf, AssetError> {
    if let Ok(path) = env::var(ASSETS_OVERRIDE_ENV) {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(AssetError::AssetsRootNotFound {
            reason: format!("{ASSETS_OVERRIDE_ENV} points to a missing directory"),
        });
    }

    if let Some(path) = assets_next_to_executable() {
        return Ok(path);
    }

    #[cfg(debug_assertions)]
    if let Some(path) = manifest_assets_root() {
        return Ok(path);
    }

    Err(AssetError::AssetsRootNotFound {
        reason: format!(
            "expected `{ASSETS_DIR_NAME}` next to the executable \
             (or set {ASSETS_OVERRIDE_ENV})"
        ),
    })
}

fn assets_next_to_executable() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let assets = exe.parent()?.join(ASSETS_DIR_NAME);
    assets.is_dir().then_some(assets)
}

#[cfg(debug_assertions)]
fn manifest_assets_root() -> Option<PathBuf> {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ASSETS_DIR_NAME);
    assets.is_dir().then_some(assets)
}
