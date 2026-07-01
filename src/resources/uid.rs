use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceUid<T> {
    index: u32,
    _marker: PhantomData<T>,
}

impl<T> ResourceUid<T> {
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    pub const fn index(&self) -> u32 {
        self.index
    }
}

pub type TextureUid = ResourceUid<TextureTag>;
pub type MapUid = ResourceUid<MapTag>;
pub type ShaderUid = ResourceUid<ShaderTag>;
pub type ConfigUid = ResourceUid<ConfigTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigTag;

pub struct TextureManifestEntry {
    pub uid: TextureUid,
    pub path: &'static str,
}

pub struct MapManifestEntry {
    pub uid: MapUid,
    pub path: &'static str,
}

pub struct ShaderManifestEntry {
    pub uid: ShaderUid,
    pub vert: &'static str,
    pub frag: &'static str,
}

pub struct ConfigManifestEntry {
    pub uid: ConfigUid,
    pub path: &'static str,
}
