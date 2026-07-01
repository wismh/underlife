use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::resources::asset::Asset;
use crate::resources::assets::TEXTURES;
use crate::resources::types::texture::TextureAsset;
use crate::resources::uid::{TextureTag, TextureUid};

pub struct ResourceManager {
    assets_root: PathBuf,
    textures: TypedStore<TextureTag, TextureAsset>,
}

impl ResourceManager {
    pub fn load_all() -> Self {
        let assets_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let mut textures = TypedStore::<TextureTag, TextureAsset>::with_capacity(TEXTURES.len());

        for entry in TEXTURES {
            let path = assets_root.join(entry.path);
            textures
                .insert(entry.uid, TextureAsset::load(&path).expect("load texture asset"))
                .expect("duplicate texture uid");
        }

        Self {
            assets_root,
            textures,
        }
    }

    pub fn assets_root(&self) -> &Path {
        &self.assets_root
    }

    pub fn texture(&self, uid: TextureUid) -> &TextureAsset {
        self.textures.get(uid)
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

pub type ResourceUidFor<M> = crate::resources::uid::ResourceUid<M>;
