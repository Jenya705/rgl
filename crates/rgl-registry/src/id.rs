use crate::*;

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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnknownRegistryId(RegistryIdNumeric);

impl UnknownRegistryId {
    pub const NONE: Self = Self(RegistryIdNumeric::MAX);

    pub fn new<I>() -> Self
    where
        I: RegistryItem,
    {
        Self(I::id().into())
    }

    /// If the registry of this id and the given one varies, then this method can give incorrect result
    pub fn is<I>(&self) -> bool
    where
        I: RegistryItem,
    {
        self.0 == I::id().into()
    }
}

impl<R: Registry> From<RegistryId<R>> for UnknownRegistryId {
    fn from(value: RegistryId<R>) -> Self {
        Self(value.0.into())
    }
}

impl<R: Registry> From<Option<RegistryId<R>>> for UnknownRegistryId {
    fn from(value: Option<RegistryId<R>>) -> Self {
        match value {
            Some(rid) => Self::from(rid),
            None => Self::NONE,
        }
    }
}
