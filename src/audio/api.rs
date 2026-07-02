use std::time::Duration;

use glam::Vec3;
use thiserror::Error;

use crate::resources::{SoundPresetUid, SoundUid};

pub const ONE_SHOT_POOL_SIZE: usize = 12;
pub const DEFAULT_FADE: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchRange {
    pub min: f32,
    pub max: f32,
}

impl PitchRange {
    pub const UNITY: Self = Self { min: 1.0, max: 1.0 };

    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

/// Sound preset — recipe for playback, not the clip itself.
#[derive(Debug, Clone, Copy)]
pub struct SoundData {
    pub clip: SoundUid,
    pub volume: f32,
    pub pitch: PitchRange,
}

#[derive(Debug, Clone, Copy)]
pub enum SoundSource {
    Preset(SoundPresetUid),
    Clip(SoundUid),
    Data(SoundData),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayParams {
    pub volume: Option<f32>,
    pub pitch: Option<PitchRange>,
    pub spatial: Option<SpatialParams>,
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialParams {
    pub position: Vec3,
    pub max_distance: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VolumeLevels {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
}

pub type LoopHandle = u64;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("audio backend error: {0}")]
    Backend(String),
    #[error("unknown sound clip")]
    UnknownClip,
    #[error("unknown sound preset uid {0}")]
    UnknownPreset(u32),
    #[error("invalid loop handle")]
    InvalidLoopHandle,
}
