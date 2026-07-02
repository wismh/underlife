use crate::audio::api::{AudioError, PitchRange, SoundData};
use crate::resources::assets::SOUND_PRESETS;
use crate::resources::manager::ResourceManager;
use crate::resources::SoundPresetUid;

#[derive(Debug, Default)]
pub struct SoundPresetRegistry {
    presets: Vec<Option<SoundData>>,
}

impl SoundPresetRegistry {
    pub fn from_resources(resources: &ResourceManager) -> Result<Self, AudioError> {
        let mut presets = vec![None; SOUND_PRESETS.len().max(1)];

        for entry in SOUND_PRESETS {
            let asset = resources.sound_preset(entry.uid);
            let clip = resources
                .sound_by_name(&asset.clip)
                .ok_or(AudioError::UnknownClip)?;
            let index = entry.uid.index() as usize;
            if index >= presets.len() {
                presets.resize(index + 1, None);
            }
            presets[index] = Some(SoundData {
                clip,
                volume: asset.volume,
                pitch: PitchRange::new(asset.pitch_min, asset.pitch_max),
            });
        }

        Ok(Self { presets })
    }

    pub fn get(&self, uid: SoundPresetUid) -> Option<SoundData> {
        self.presets
            .get(uid.index() as usize)
            .and_then(|preset| *preset)
    }

    pub fn resolve(&self, source: super::api::SoundSource) -> Result<SoundData, AudioError> {
        match source {
            super::api::SoundSource::Preset(uid) => self
                .get(uid)
                .ok_or(AudioError::UnknownPreset(uid.index())),
            super::api::SoundSource::Clip(clip) => Ok(SoundData {
                clip,
                volume: 1.0,
                pitch: PitchRange::UNITY,
            }),
            super::api::SoundSource::Data(data) => Ok(data),
        }
    }
}
