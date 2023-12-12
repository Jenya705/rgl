use std::any::type_name;
use std::iter;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::utils::HashMap;
use parking_lot::Mutex;

#[doc(hidden)]
pub mod __private {
    pub use ctor;
    pub use parking_lot;
    pub use paste;
}

pub struct RegistryPlugin<R: Registry>(PhantomData<R>);

impl<R: Registry> Default for RegistryPlugin<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<R: Registry> Plugin for RegistryPlugin<R> {
    fn build(&self, app: &mut App) {
        app.init_resource::<RegistryData<R>>();

        #[cfg(debug_assertions)]
        app.add_systems(PostStartup, check_unregistered::<R>);
    }
}

fn check_unregistered<R: Registry>(registry: Res<RegistryData<R>>) {
    registry.unregistered().for_each(|i| {
        warn!(
            "Unregistered item in registry {}, item index: {}",
            type_name::<R>(),
            i
        );
    });
}

pub trait RegistryItem: Sync + Send + 'static {
    type R: Registry;

    fn id() -> <Self::R as Registry>::Id;
}

pub trait Registry: Sync + Send + 'static {
    type Id: PartialEq + Clone + Sync + Send + Into<usize> + 'static;

    fn next_id() -> &'static Mutex<Self::Id>;
}

pub trait ChildRegistry: Registry + RegistryItem<R = RegistriesRegistry> {}

#[repr(transparent)]
pub struct RegistryId<R: Registry>(R::Id);

impl<R: Registry> RegistryId<R> {
    pub fn from_id(id: R::Id) -> Self {
        Self(id)
    }

    pub fn from_item<I: RegistryItem<R = R>>() -> Self {
        Self(I::id())
    }

    pub fn is<I: RegistryItem<R = R>>(&self) -> bool {
        self.0 == I::id()
    }
}

impl<R: Registry> PartialEq for RegistryId<R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<R: Registry> Clone for RegistryId<R> {
    fn clone(&self) -> Self {
        Self::from_id(self.0.clone())
    }
}

impl<R: Registry> Copy for RegistryId<R> where R::Id: Copy {}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct UnknownRegistryId(pub usize);

impl UnknownRegistryId {
    pub const NONE: Self = Self(usize::MAX);

    pub fn from_option<R: Registry>(value: Option<RegistryId<R>>) -> Self {
        match value {
            Some(rid) => rid.into(),
            None => Self::NONE,
        }
    }
}

impl<R: Registry> From<RegistryId<R>> for UnknownRegistryId {
    fn from(value: RegistryId<R>) -> Self {
        Self(value.0.into())
    }
}

#[derive(Resource, Debug)]
pub struct RegistryData<R: Registry> {
    names2id: HashMap<&'static str, R::Id>,
    id2names: Vec<Option<&'static str>>,
}

impl<R: Registry> RegistryData<R> {
    pub fn add(&mut self, name: &'static str, id: R::Id) {
        self.names2id.insert(name, id.clone());
        let id = id.into();
        if self.id2names.len() <= id {
            self.id2names.resize(id + 1, None);
        }
        self.id2names[id] = Some(name);
    }

    pub fn name(&self, id: R::Id) -> Option<&'static str> {
        self.id2names.get(id.into()).cloned().flatten()
    }

    pub fn id(&self, name: &str) -> Option<R::Id> {
        self.names2id.get(name).cloned()
    }

    pub fn unregistered(&self) -> impl Iterator<Item = usize> + '_ {
        let len: usize = R::next_id().lock().clone().into();

        self.id2names
            .iter()
            .cloned()
            .chain(iter::repeat(None).take(len - self.id2names.len()))
            .enumerate()
            .filter(|(_, o)| o.is_none())
            .map(|(i, _)| i)
    }

    pub fn remove(&mut self, id: R::Id) -> bool {
        let id: usize = id.into();
        if let Some(Some(ref name)) = self.id2names.get(id) {
            self.names2id.remove(name);
            self.id2names[id] = None;
            true
        } else {
            false
        }
    }
}

impl<R: Registry> Default for RegistryData<R> {
    fn default() -> Self {
        Self {
            names2id: HashMap::new(),
            id2names: Vec::new(),
        }
    }
}

pub trait RegistryAppMethods {
    fn register_item<I: RegistryItem>(&mut self, name: &'static str) -> &mut Self;
}

impl RegistryAppMethods for App {
    fn register_item<I: RegistryItem>(&mut self, name: &'static str) -> &mut Self {
        self.init_resource::<RegistryData<I::R>>();
        self.world
            .resource_mut::<RegistryData<I::R>>()
            .add(name, I::id());
        self
    }
}

pub static REGISTRY_NEXT_ID: Mutex<usize> = Mutex::new(0);

/// A special registry, that handles ids of registries. The only registry that can have none id
pub struct RegistriesRegistry;

impl Registry for RegistriesRegistry {
    type Id = usize;

    fn next_id() -> &'static Mutex<Self::Id> {
        &REGISTRY_NEXT_ID
    }
}

#[macro_export]
macro_rules! new_registry {
    ($registry: ident, $id: ty) => {
        pub struct $registry;

        $crate::__private::paste::paste! {
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static [<__ $registry _NEXT_ID>]: $crate::__private::parking_lot::Mutex<$id> =
                $crate::__private::parking_lot::Mutex::<$id>::new(0);

        }

        impl $crate::Registry for $registry {
            type Id = $id;

            fn next_id() -> &'static $crate::__private::parking_lot::Mutex<$id> {
                $crate::__private::paste::paste! {
                    &[<__ $registry _NEXT_ID>]
                }
            }
        }

        $crate::__impl_registry_item!($registry, $crate::RegistriesRegistry);

        impl $crate::ChildRegistry for $registry {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_registry_item {
    ($item: ident, $registry: ty) => {
        $crate::__private::paste::paste! {
            #[doc(hidden)]
            #[$crate::__private::ctor::ctor]
            #[allow(non_upper_case_globals)]
            static [<__ $item _ID>]: <$registry as $crate::Registry>::Id = {
                let mut lock = <$registry as $crate::Registry>::next_id().lock();
                let id = *lock;
                *lock += 1;
                id
            };
        }

        impl $crate::RegistryItem for $item {
            type R = $registry;

            fn id() -> <Self::R as $crate::Registry>::Id {
                $crate::__private::paste::paste! {
                    *[<__ $item _ID>]
                }
            }
        }
    };
}

#[macro_export]
macro_rules! new_registry_item {
    ($item: ident, $registry: ty) => {
        pub struct $item;

        $crate::__impl_registry_item!($item, $registry);
    };
}

#[cfg(test)]
mod tests {

    use super::*;

    new_registry_item!(TestItem1, TestRegistry);
    new_registry!(TestRegistry, u8);
    new_registry_item!(TestItem2, TestRegistry);

    #[test]
    fn easy_test() {
        assert!(
            TestItem1::id() == 0 || TestItem2::id() == 0,
            "1: {}, 2: {}",
            TestItem1::id(),
            TestItem2::id()
        );
        assert!(
            TestItem1::id() == 1 || TestItem2::id() == 1,
            "1: {}, 2: {}",
            TestItem1::id(),
            TestItem2::id()
        );
        assert_ne!(TestItem1::id(), TestItem2::id());
    }

    #[test]
    fn bevy_test() {
        let mut app = App::new();

        app.add_plugins(RegistryPlugin::<TestRegistry>::default())
            .register_item::<TestItem1>("Test1");

        let registry_data = app.world.resource::<RegistryData<TestRegistry>>();

        assert_eq!(
            registry_data.unregistered().collect::<Vec<_>>(),
            vec![TestItem2::id() as usize]
        );

        assert_eq!(registry_data.id("Test1"), Some(TestItem1::id()),);
        assert_eq!(registry_data.name(TestItem1::id()), Some("Test1"));
        assert_eq!(registry_data.id("Test2"), None);
        assert_eq!(registry_data.name(TestItem2::id()), None);
    }
}
