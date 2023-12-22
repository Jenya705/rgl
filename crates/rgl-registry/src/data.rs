use std::{
    any::type_name,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use bevy::{ecs::system::Resource, log::error, utils::hashbrown::HashMap};

use crate::{id::RegistryId, Registry, RegistryItem};

pub trait RegistryMapInsert<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V>;
}

pub trait RegistryMapGet<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
}

pub trait RegistryMapGetMut<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
}

pub trait RegistryMapConvert {
    type Converted;

    fn convert(self) -> Self::Converted;
}

impl<K, V, S> RegistryMapInsert<K, V> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
}

impl<K, V, S> RegistryMapGet<K, V> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
}

impl<K, V, S> RegistryMapGetMut<K, V> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }
}

impl<K, V, S> RegistryMapConvert for HashMap<K, V, S> {
    type Converted = HashMap<K, V, S>;

    fn convert(self) -> Self::Converted {
        self
    }
}

pub struct RegistryIdMap<R: Registry, V> {
    objects: Vec<Option<V>>,
    _marker: PhantomData<R>,
}

impl<R: Registry, V> RegistryMapInsert<RegistryId<R>, V> for RegistryIdMap<R, V> {
    fn insert(&mut self, key: RegistryId<R>, value: V) -> Option<V> {
        let index = key.numeric() as usize;
        if index >= self.objects.len() {
            self.objects.resize_with(index + 1, || None);
        }
        self.objects
            .get_mut(index)
            .map(|it| std::mem::replace(it, Some(value)))
            .flatten()
    }
}

impl<R: Registry, V> RegistryMapGet<RegistryId<R>, V> for RegistryIdMap<R, V> {
    fn get(&self, key: &RegistryId<R>) -> Option<&V> {
        self.objects
            .get(key.clone().numeric() as usize)
            .map(Option::as_ref)
            .flatten()
    }
}

impl<R: Registry, V> RegistryMapGetMut<RegistryId<R>, V> for RegistryIdMap<R, V> {
    fn get_mut(&mut self, key: &RegistryId<R>) -> Option<&mut V> {
        self.objects
            .get_mut(key.clone().numeric() as usize)
            .map(Option::as_mut)
            .flatten()
    }
}

impl<R: Registry, V> RegistryMapConvert for RegistryIdMap<R, V> {
    type Converted = ConvertedRegistryIdMap<R, V>;

    fn convert(self) -> Self::Converted {
        let mut any_missing = false;

        self.objects
            .iter()
            .chain(std::iter::repeat(&None).take(R::count() - self.objects.len()))
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .for_each(|(i, _)| {
                error!(
                    "There is no binding with id {} in RegistryMap for registry {}",
                    i,
                    type_name::<R>()
                );

                any_missing = true;
            });

        if any_missing {
            panic!("RegistryMap couldn't be finished");
        }

        ConvertedRegistryIdMap {
            objects: self.objects.into_iter().map(Option::unwrap).collect(),
            _marker: PhantomData,
        }
    }
}

impl<R: Registry, V> RegistryIdMap<R, V> {
    pub fn iter(&self) -> impl Iterator<Item = &V> + '_ {
        self.objects.iter().filter_map(Option::as_ref)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.objects.iter_mut().filter_map(Option::as_mut)
    }
}

impl<R: Registry, V> Default for RegistryIdMap<R, V> {
    fn default() -> Self {
        Self {
            objects: vec![],
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct ConvertedRegistryIdMap<R: Registry, V> {
    objects: Vec<V>,
    _marker: PhantomData<R>,
}

impl<R: Registry, V> RegistryMapGet<RegistryId<R>, V> for ConvertedRegistryIdMap<R, V> {
    fn get(&self, key: &RegistryId<R>) -> Option<&V> {
        self.objects.get(key.clone().numeric() as usize)
    }
}

impl<R: Registry, V> RegistryMapGetMut<RegistryId<R>, V> for ConvertedRegistryIdMap<R, V> {
    fn get_mut(&mut self, key: &RegistryId<R>) -> Option<&mut V> {
        self.objects.get_mut(key.clone().numeric() as usize)
    }
}

impl<R: Registry, V> RegistryMapInsert<RegistryId<R>, V> for ConvertedRegistryIdMap<R, V> {
    fn insert(&mut self, key: RegistryId<R>, value: V) -> Option<V> {
        let index = key.numeric() as usize;
        if self.objects.len() <= index {
            panic!("ConvertedRegistryIdMap doesn't support creating new values");
        }
        Some(std::mem::replace(&mut self.objects[index], value))
    }
}

impl<R: Registry, V> ConvertedRegistryIdMap<R, V> {
    pub fn iter(&self) -> core::slice::Iter<'_, V> {
        self.objects.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, V> {
        self.objects.iter_mut()
    }
}

impl RegistryMapConvert for () {
    type Converted = Self;

    fn convert(self) -> Self::Converted {
        self
    }
}

/// A table with 2 keys, that point to each other. The second key (named value here) should be not RegistryId
pub type RegistryTwoSidedDataCellId2Value<R, V> = RegistryDataCell<
    RegistryId<R>,
    V,
    <RegistryIdMap<R, V> as RegistryMapConvert>::Converted,
    <HashMap<V, RegistryId<R>> as RegistryMapConvert>::Converted,
>;

/// A table with 2 keys, that point to each other. The second key (named value here) will be RegistryId
pub type RegistryTwoSidedDataCellId2Id<R1, R2> = RegistryDataCell<
    RegistryId<R1>,
    RegistryId<R2>,
    <RegistryIdMap<R1, RegistryId<R2>> as RegistryMapConvert>::Converted,
    <RegistryIdMap<R2, RegistryId<R1>> as RegistryMapConvert>::Converted,
>;

/// A table with 1 key and 1 extra column, key points to one column
pub type RegistryOneSidedDataCell<R, V> =
    RegistryDataCell<RegistryId<R>, V, <RegistryIdMap<R, V> as RegistryMapConvert>::Converted>;

/// The same as [`RegistryTwoSidedDataCellId2Value`] but will not be optimized after Startup phase,
/// that doesn't require all keys to be filled and new keys can be added in non Startup phases
pub type ChangableRegistryTwoSidedDataCellId2Value<R, V> =
    RegistryDataCell<RegistryId<R>, V, RegistryIdMap<R, V>, HashMap<V, RegistryId<R>>>;

/// The same as [`RegistryTwoSidedDataCellId2Id`] with concepts of [`ChangableRegistryTwoSidedDataCellId2Value`]
pub type ChangableRegistryTwoSidedDataCellId2Id<R1, R2> = RegistryDataCell<
    RegistryId<R1>,
    RegistryId<R2>,
    RegistryIdMap<R1, RegistryId<R2>>,
    RegistryIdMap<R2, RegistryId<R1>>,
>;

/// The same as [`RegistryOneSidedDataCell`] with concepts of [`ChangableRegistryTwoSidedDataCellId2Value`]
pub type ChangableRegistryOneSidedDataCell<R, V> =
    RegistryDataCell<RegistryId<R>, V, RegistryIdMap<R, V>>;

#[doc(hidden)]
#[derive(Resource)]
pub struct RegistryDataCell<I, V, C1, C2 = ()> {
    pub c1: C1,
    pub c2: C2,
    _marker: PhantomData<(I, V)>,
}

impl<Id, Value, C1, C2> RegistryDataCell<Id, Value, C1, C2> {
    /// Returns value using given id
    pub fn value(&self, id: &Id) -> Option<&Value>
    where
        C1: RegistryMapGet<Id, Value>,
    {
        self.c1.get(id)
    }

    /// Returns value using given value
    pub fn id(&self, value: &Value) -> Option<&Id>
    where
        C2: RegistryMapGet<Value, Id>,
    {
        self.c2.get(value)
    }

    /// Inserts new entry
    pub fn insert(&mut self, id: Id, value: Value) -> (Option<Value>, Option<Id>)
    where
        C1: RegistryMapInsert<Id, Value>,
        C2: RegistryMapInsert<Value, Id>,
        Value: Clone,
        Id: Clone,
    {
        (
            self.c1.insert(id.clone(), value.clone()),
            self.c2.insert(value.clone(), id.clone()),
        )
    }

    /// Converts this registry data cell to more optimized one
    pub fn convert(self) -> RegistryDataCell<Id, Value, C1::Converted, C2::Converted>
    where
        C1: RegistryMapConvert,
        C2: RegistryMapConvert,
    {
        RegistryDataCell {
            c1: self.c1.convert(),
            c2: self.c2.convert(),
            _marker: PhantomData,
        }
    }
}

impl<R, Value, C1, C2> RegistryDataCell<RegistryId<R>, Value, C1, C2>
where
    R: Registry,
{
    /// Returns value using given [`RegistryItem`]
    pub fn value_ty<I>(&self) -> Option<&Value>
    where
        I: RegistryItem<Registry = R>,
        C1: RegistryMapGet<RegistryId<R>, Value>,
    {
        self.c1.get(&RegistryId::new::<I>())
    }
}

impl<Id, Value, C1> RegistryDataCell<Id, Value, C1, ()> {
    /// Returns mutable reference to the value under given id
    pub fn value_mut(&mut self, id: &Id) -> Option<&mut Value>
    where
        C1: RegistryMapGetMut<Id, Value>,
    {
        self.c1.get_mut(id)
    }

    /// Inserts new entry, if it didn't exist and returns mutable reference to the value
    pub fn value_mut_or_insert_with(&mut self, id: &Id, with: impl FnOnce() -> Value) -> &mut Value
    where
        Id: Clone,
        C1: RegistryMapGetMut<Id, Value>,
        C1: RegistryMapInsert<Id, Value>,
    {
        // TODO: Some issues with borrow checker
        if self.c1.get_mut(id).is_none() {
            self.c1.insert(id.clone(), with());
        }

        self.c1.get_mut(id).unwrap()
    }

    /// Inserts new entry, if it didn't exist and returns mutable reference to the value
    pub fn value_mut_or_insert_default(&mut self, id: &Id) -> &mut Value
    where
        Id: Clone,
        Value: Default,
        C1: RegistryMapGetMut<Id, Value>,
        C1: RegistryMapInsert<Id, Value>,
    {
        self.value_mut_or_insert_with(id, Default::default)
    }

    /// Inserts new entry
    pub fn insert_one_sided(&mut self, id: Id, value: Value) -> Option<Value>
    where
        C1: RegistryMapInsert<Id, Value>,
    {
        self.c1.insert(id, value)
    }
}

impl<I, V, C1, C2> Default for RegistryDataCell<I, V, C1, C2>
where
    C1: Default,
    C2: Default,
{
    fn default() -> Self {
        Self {
            c1: C1::default(),
            c2: C2::default(),
            _marker: PhantomData,
        }
    }
}
