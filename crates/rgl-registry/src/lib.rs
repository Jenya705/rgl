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
    type Id: Clone + Sync + Send + Into<usize> + 'static;

    fn next_id() -> &'static Mutex<Self::Id>;
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
    };
}

#[macro_export]
macro_rules! new_item {
    ($item: ident, $registry: ty) => {
        pub struct $item;

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

#[cfg(test)]
mod tests {

    use super::*;

    new_item!(TestItem1, TestRegistry);
    new_registry!(TestRegistry, u8);
    new_item!(TestItem2, TestRegistry);

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
