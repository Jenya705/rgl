use crate::*;

/// A container for registry's id. Helps not to mix up different registries' ids
#[repr(transparent)]
pub struct RegistryId<R: Registry>(R::Id);

impl<R: Registry> RegistryId<R> {
    pub fn new<I>() -> Self
    where
        I: RegistryItem<Registry = R>,
    {
        Self(I::id())
    }

    pub fn is<I>(&self) -> bool
    where
        I: RegistryItem<Registry = R>,
    {
        self.0 == I::id()
    }

    pub fn numeric(self) -> RegistryIdNumeric {
        self.0.into()
    }

    pub fn iter_all() -> impl Iterator<Item = RegistryId<R>> {
        R::iter_all().map(|id| Self(id))
    }
}

impl<R: Registry> PartialEq for RegistryId<R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<R: Registry> Eq for RegistryId<R> {}

impl<R: Registry> Clone for RegistryId<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Registry> Copy for RegistryId<R> where R::Id: Copy {}
