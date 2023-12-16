mod app;
mod data;
mod id;

pub use app::*;
pub use data::*;
pub use id::*;

type RegistryIdNumeric = u16;

pub trait Registry: 'static + Sync + Send + Sized {
    type Id: 'static + Clone + Sync + Send + Sized + PartialEq + Eq + Into<RegistryIdNumeric>;

    fn reserve_id() -> Self::Id;

    fn count() -> usize;
}

pub trait RegistryItem: 'static + Sync + Send + Sized {
    type Registry: Registry;

    fn id() -> <Self::Registry as Registry>::Id;
}

pub trait ChildRegistry: Registry + RegistryItem<Registry = Registries> {}

pub struct Registries;

__registry_impl!(Registries, RegistryIdNumeric);

#[doc(hidden)]
pub mod __private {
    pub use ctor;
    pub use parking_lot;
    pub use paste;
}

#[doc(hidden)]
#[macro_export]
macro_rules! __registry_item_impl {
    ($item: ident, $registry: ty) => {
        $crate::__private::paste::paste! {
            #[doc(hidden)]
            #[$crate::__private::ctor::ctor]
            #[allow(non_upper_case_globals)]
            static [<__ $item _ID>]: <$registry as $crate::Registry>::Id =
                <$registry as $crate::Registry>::reserve_id();
        }

        impl $crate::RegistryItem for $item {
            type Registry = $registry;

            fn id() -> <Self::Registry as $crate::Registry>::Id {
                $crate::__private::paste::paste! {
                    *[<__ $item _ID>]
                }
            }
        }
    };
}

#[macro_export]
macro_rules! new_registry_items {
    ($registry: ty {$($item: ident$(,)?)*}) => {
        $(
            pub struct $item;

            $crate::__registry_item_impl!($item, $registry);
        )*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __registry_impl {
    ($registry: ident, $id: ty) => {
        $crate::__private::paste::paste! {
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static [<__ $registry _ID_COUNTER>]: $crate::__private::parking_lot::Mutex<$id> =
                $crate::__private::parking_lot::Mutex::new(0);
        }

        impl $crate::Registry for $registry {
            type Id = $id;

            fn reserve_id() -> Self::Id {
                $crate::__private::paste::paste! {
                    let mut lock = [<__ $registry _ID_COUNTER>].lock();
                    let id = *lock;
                    *lock += 1;
                    id
                }
            }

            fn count() -> usize {
                $crate::__private::paste::paste! {
                    (*[<__ $registry _ID_COUNTER>].lock()).into()
                }
            }
        }
    };
}

#[macro_export]
macro_rules! new_registry {
    ($registry: ident, $id: ty) => {
        pub struct $registry;

        $crate::__registry_impl!($registry, $id);
        $crate::__registry_item_impl!($registry, $crate::Registries);

        impl $crate::ChildRegistry for $registry {}
    };
}

#[cfg(test)]
mod tests {

    use bevy::{app::App, log::LogPlugin};

    use super::*;

    new_registry!(TestRegistry, u8);
    new_registry_items!(TestRegistry {
        TestItem1,
        TestItem2,
        TestItem3,
    });

    #[test]
    fn test() {
        let mut app = App::new();
        app.add_plugins(LogPlugin::default())
            .register_two_sided_data_id2value::<TestItem1, &'static str>("test_1")
            .register_two_sided_data_id2value::<TestItem2, &'static str>("test_2")
            .register_two_sided_data_id2value::<TestItem3, &'static str>("test_3")
            .register_one_sided_data::<TestItem1, u32>(1)
            .register_one_sided_data::<TestItem2, u32>(2)
            .register_one_sided_data::<TestItem3, u32>(3);
        app.update();

        let res = app
            .world
            .resource::<RegistryTwoSidedDataCellId2Value<TestRegistry, &'static str>>();

        assert_eq!(res.value_ty::<TestItem1>(), Some(&"test_1"));
        assert_eq!(res.value_ty::<TestItem2>(), Some(&"test_2"));
        assert_eq!(res.value_ty::<TestItem3>(), Some(&"test_3"));

        let res = app
            .world
            .resource::<RegistryOneSidedDataCell<TestRegistry, u32>>();

        assert_eq!(res.value_ty::<TestItem1>(), Some(&1));
        assert_eq!(res.value_ty::<TestItem2>(), Some(&2));
        assert_eq!(res.value_ty::<TestItem3>(), Some(&3));
    }
}
