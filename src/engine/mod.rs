mod app;
mod window;

use thiserror::Error;

use crate::resources::AssetError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Assets(#[from] AssetError),
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),
}

pub use app::run;
pub use window::{EngineConfig, WindowContext};
