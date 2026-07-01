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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureTag;

pub struct TextureManifestEntry {
    pub uid: TextureUid,
    pub path: &'static str,
}
