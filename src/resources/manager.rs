use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::resources::asset::{Asset, AssetError};
use crate::resources::assets::{CONFIGS, MAPS, SHADERS, SOUNDS, SOUND_PRESETS, TEXTURES};
use crate::resources::paths;
use crate::resources::types::config::ConfigAsset;
use crate::resources::types::map::MapAsset;
use crate::resources::types::shader::ShaderAsset;
use crate::resources::types::sound::SoundAsset;
use crate::resources::types::sound_preset::SoundPresetAsset;
use crate::resources::types::texture::TextureAsset;
use crate::resources::uid::{
    ConfigTag, ConfigUid, MapTag, MapUid, ShaderTag, ShaderUid, SoundPresetTag, SoundPresetUid,
    SoundTag, SoundUid, TextureTag, TextureUid,
};

pub struct ResourceManager {
    assets_root: PathBuf,
    textures: TypedStore<TextureTag, TextureAsset>,
    maps: TypedStore<MapTag, MapAsset>,
    shaders: TypedStore<ShaderTag, ShaderAsset>,
    configs: TypedStore<ConfigTag, ConfigAsset>,
    sounds: TypedStore<SoundTag, SoundAsset>,
    sound_presets: TypedStore<SoundPresetTag, SoundPresetAsset>,
}

impl ResourceManager {
    pub fn load_all() -> Result<Self, AssetError> {
        let assets_root = paths::resolve_assets_root()?;
        let mut textures = TypedStore::<TextureTag, TextureAsset>::with_capacity(TEXTURES.len());
        let mut maps = TypedStore::<MapTag, MapAsset>::with_capacity(MAPS.len());
        let mut shaders = TypedStore::<ShaderTag, ShaderAsset>::with_capacity(SHADERS.len());
        let mut configs = TypedStore::<ConfigTag, ConfigAsset>::with_capacity(CONFIGS.len());
        let mut sounds = TypedStore::<SoundTag, SoundAsset>::with_capacity(SOUNDS.len());
        let mut sound_presets =
            TypedStore::<SoundPresetTag, SoundPresetAsset>::with_capacity(SOUND_PRESETS.len());

        for entry in TEXTURES {
            let path = assets_root.join(entry.path);
            insert_unique(
                &mut textures,
                entry.uid,
                TextureAsset::load(&path)?,
                "texture",
            )?;
        }

        for entry in MAPS {
            let path = assets_root.join(entry.path);
            insert_unique(&mut maps, entry.uid, MapAsset::load(&path)?, "map")?;
        }

        for entry in SHADERS {
            insert_unique(
                &mut shaders,
                entry.uid,
                ShaderAsset::load_pair(
                    &assets_root.join(entry.vert),
                    &assets_root.join(entry.frag),
                )?,
                "shader",
            )?;
        }

        for entry in CONFIGS {
            let path = assets_root.join(entry.path);
            insert_unique(&mut configs, entry.uid, ConfigAsset::load(&path)?, "config")?;
        }

        for entry in SOUNDS {
            let path = assets_root.join(entry.path);
            insert_unique(&mut sounds, entry.uid, SoundAsset::load(&path)?, "sound")?;
        }

        for entry in SOUND_PRESETS {
            let path = assets_root.join(entry.path);
            insert_unique(
                &mut sound_presets,
                entry.uid,
                SoundPresetAsset::load(&path)?,
                "sound_preset",
            )?;
        }

        Ok(Self {
            assets_root,
            textures,
            maps,
            shaders,
            configs,
            sounds,
            sound_presets,
        })
    }

    pub fn assets_root(&self) -> &Path {
        &self.assets_root
    }

    pub fn texture(&self, uid: TextureUid) -> &TextureAsset {
        self.textures.get(uid)
    }

    pub fn map(&self, uid: MapUid) -> &MapAsset {
        self.maps.get(uid)
    }

    pub fn shader(&self, uid: ShaderUid) -> &ShaderAsset {
        self.shaders.get(uid)
    }

    pub fn config(&self, uid: ConfigUid) -> &ConfigAsset {
        self.configs.get(uid)
    }

    pub fn sound(&self, uid: SoundUid) -> &SoundAsset {
        self.sounds.get(uid)
    }

    pub fn sound_preset(&self, uid: SoundPresetUid) -> &SoundPresetAsset {
        self.sound_presets.get(uid)
    }
}

pub struct TypedStore<M, T> {
    entries: Vec<Option<T>>,
    _marker: PhantomData<M>,
}

impl<M, T> TypedStore<M, T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: (0..capacity).map(|_| None).collect(),
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, uid: ResourceUidFor<M>, value: T) -> Result<(), T>
    where
        M: ResourceMarker,
    {
        let index = uid.index() as usize;
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        if self.entries[index].is_some() {
            return Err(value);
        }

        self.entries[index] = Some(value);
        Ok(())
    }

    pub fn get(&self, uid: ResourceUidFor<M>) -> &T
    where
        M: ResourceMarker,
    {
        let index = uid.index();
        self.try_get(uid)
            .unwrap_or_else(|| panic!("missing resource uid {index}"))
    }

    pub fn try_get(&self, uid: ResourceUidFor<M>) -> Option<&T>
    where
        M: ResourceMarker,
    {
        self.entries.get(uid.index() as usize)?.as_ref()
    }
}

pub trait ResourceMarker {}

impl ResourceMarker for TextureTag {}
impl ResourceMarker for MapTag {}
impl ResourceMarker for ShaderTag {}
impl ResourceMarker for ConfigTag {}
impl ResourceMarker for SoundTag {}
impl ResourceMarker for SoundPresetTag {}

pub type ResourceUidFor<M> = crate::resources::uid::ResourceUid<M>;

fn insert_unique<M, T>(
    store: &mut TypedStore<M, T>,
    uid: ResourceUidFor<M>,
    value: T,
    kind: &'static str,
) -> Result<(), AssetError>
where
    M: ResourceMarker,
{
    let index = uid.index();
    store
        .insert(uid, value)
        .map_err(|_| AssetError::DuplicateUid { kind, index })
}
