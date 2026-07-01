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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderTag;

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
