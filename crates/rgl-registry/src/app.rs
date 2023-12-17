use std::{any::TypeId, hash::Hash};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    Registry, RegistryDataCell, RegistryId, RegistryIdMap, RegistryItem, RegistryMapConvert,
    RegistryMapInsert,
};

pub trait RegistryAppExt {
    fn register_two_sided_data_id2id<I1, I2>(&mut self) -> &mut Self
    where
        I1: RegistryItem,
        I2: RegistryItem;

    fn register_two_sided_data_id2value<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Hash,
        V: Clone,
        V: Sync + Send + 'static,
        V: Eq;

    fn register_one_sided_data<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Sync + Send + 'static;

    fn register_one_sided_vec_data<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Sync + Send + 'static;

    fn keep_changable_two_sided_data_id2id<R1, R2>(&mut self) -> &mut Self
    where
        R1: Registry,
        R2: Registry;

    fn keep_changable_two_sided_data_id2value<R, V>(&mut self) -> &mut Self
    where
        R: Registry,
        V: Hash,
        V: Clone,
        V: Sync + Send + 'static,
        V: Eq;

    fn keep_changable_one_sided_data<R, V>(&mut self) -> &mut Self
    where
        R: Registry,
        V: Sync + Send + 'static;
}

#[derive(Resource, Default)]
struct ConvertSystemsData {
    added_systems: HashSet<TypeId>,
    cancelled_systems: HashSet<TypeId>,
}

impl ConvertSystemsData {
    pub fn is_added<T: 'static>(&self, _marker: &T) -> bool {
        self.added_systems.contains(&TypeId::of::<T>())
    }

    pub fn add_if_not_added<T: IntoSystemConfigs<M> + 'static, M>(
        app: &mut App,
        func: T,
        pre: bool,
    ) {
        let mut acs = app.world.remove_resource::<ConvertSystemsData>().unwrap();
        if !acs.is_added(&func) {
            if pre {
                app.add_systems(PreStartup, func);
            } else {
                app.add_systems(Startup, func);
            }
            acs.added_systems.insert(TypeId::of::<T>());
        }
        app.insert_resource(acs);
    }

    pub fn cancel<T: 'static>(&mut self, _marker: T) {
        self.cancelled_systems.insert(TypeId::of::<T>());
    }

    pub fn is_cancelled<T: 'static>(&self, _marker: T) -> bool {
        self.cancelled_systems.contains(&TypeId::of::<T>())
    }
}

fn convert_system<I, V, C1, C2>(mut commands: Commands)
where
    I: Sync + Send + 'static,
    V: Sync + Send + 'static,
    C1: Sync + Send + 'static,
    C2: Sync + Send + 'static,
    C1::Converted: Sync + Send + 'static,
    C2::Converted: Sync + Send + 'static,
    C1: RegistryMapConvert,
    C2: RegistryMapConvert,
{
    commands.add(|world: &mut World| {
        if !world
            .resource::<ConvertSystemsData>()
            .is_cancelled(convert_system::<I, V, C1, C2>)
        {
            if let Some(data_cell) = world.remove_resource::<RegistryDataCell<I, V, C1, C2>>() {
                world.insert_resource(data_cell.convert());
            }
        }
    });
}

fn remove_added_convert_systems_res(mut commands: Commands) {
    commands.remove_resource::<ConvertSystemsData>();
}

fn csd_setup(app: &mut App) {
    app.world.init_resource::<ConvertSystemsData>();
    ConvertSystemsData::add_if_not_added(app, remove_added_convert_systems_res, false);
}

fn insert<I, V, C1, C2>(app: &mut App, id: I, value: V)
where
    I: Sync + Send + 'static,
    V: Sync + Send + 'static,
    C1: Sync + Send + 'static,
    C2: Sync + Send + 'static,
    C1::Converted: Sync + Send + 'static,
    C2::Converted: Sync + Send + 'static,
    I: Clone,
    V: Clone,
    C1: Default,
    C2: Default,
    C1: RegistryMapConvert,
    C2: RegistryMapConvert,
    C1: RegistryMapInsert<I, V>,
    C2: RegistryMapInsert<V, I>,
{
    ConvertSystemsData::add_if_not_added(app, convert_system::<I, V, C1, C2>, true);
    app.init_resource::<RegistryDataCell<I, V, C1, C2>>();
    // TODO: Debug?
    let _ = app
        .world
        .resource_mut::<RegistryDataCell<I, V, C1, C2>>()
        .insert(id, value);
}

fn insert_one_sided<I, V, C1>(app: &mut App, id: I, value: V)
where
    I: Sync + Send + 'static,
    V: Sync + Send + 'static,
    C1: Sync + Send + 'static,
    C1::Converted: Sync + Send + 'static,
    C1: Default,
    C1: RegistryMapConvert,
    C1: RegistryMapInsert<I, V>,
{
    ConvertSystemsData::add_if_not_added(app, convert_system::<I, V, C1, ()>, true);
    app.init_resource::<RegistryDataCell<I, V, C1, ()>>();
    // TODO: Debug?
    let _ = app
        .world
        .resource_mut::<RegistryDataCell<I, V, C1, ()>>()
        .insert_one_sided(id, value);
}

impl RegistryAppExt for App {
    fn register_two_sided_data_id2id<I1, I2>(&mut self) -> &mut Self
    where
        I1: RegistryItem,
        I2: RegistryItem,
    {
        csd_setup(self);
        insert::<
            RegistryId<I1::Registry>,
            RegistryId<I2::Registry>,
            RegistryIdMap<I1::Registry, RegistryId<I2::Registry>>,
            RegistryIdMap<I2::Registry, RegistryId<I1::Registry>>,
        >(self, RegistryId::new::<I1>(), RegistryId::new::<I2>());
        self
    }

    fn register_two_sided_data_id2value<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Hash,
        V: Clone,
        V: Sync + Send + 'static,
        V: Eq,
    {
        csd_setup(self);
        insert::<
            RegistryId<I::Registry>,
            V,
            RegistryIdMap<I::Registry, V>,
            HashMap<V, RegistryId<I::Registry>>,
        >(self, RegistryId::new::<I>(), value);
        self
    }

    fn register_one_sided_data<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Sync + Send + 'static,
    {
        csd_setup(self);
        insert_one_sided::<RegistryId<I::Registry>, V, RegistryIdMap<I::Registry, V>>(
            self,
            RegistryId::new::<I>(),
            value,
        );
        self
    }

    fn register_one_sided_vec_data<I, V>(&mut self, value: V) -> &mut Self
    where
        I: RegistryItem,
        V: Sync + Send + 'static,
    {
        csd_setup(self);
        self.init_resource::<RegistryDataCell<RegistryId<I::Registry>, Vec<V>, RegistryIdMap<I::Registry, Vec<V>>>>();
        ConvertSystemsData::add_if_not_added(
            self,
            convert_system::<RegistryId<I::Registry>, V, RegistryIdMap<I::Registry, Vec<V>>, ()>,
            true,
        );
        self.world
            .resource_mut::<RegistryDataCell<RegistryId<I::Registry>, Vec<V>, RegistryIdMap<I::Registry, Vec<V>>>>()
            .value_mut_or_insert_default(&RegistryId::new::<I>())
            .push(value);
        self
    }

    fn keep_changable_two_sided_data_id2id<R1, R2>(&mut self) -> &mut Self
    where
        R1: Registry,
        R2: Registry,
    {
        csd_setup(self);
        self.world.resource_mut::<ConvertSystemsData>().cancel(
            convert_system::<
                RegistryId<R1>,
                RegistryId<R2>,
                RegistryIdMap<R1, RegistryId<R2>>,
                RegistryIdMap<R2, RegistryId<R1>>,
            >,
        );
        self
    }

    fn keep_changable_two_sided_data_id2value<R, V>(&mut self) -> &mut Self
    where
        R: Registry,
        V: Hash,
        V: Clone,
        V: Sync + Send + 'static,
        V: Eq,
    {
        csd_setup(self);
        self.world.resource_mut::<ConvertSystemsData>().cancel(
            convert_system::<RegistryId<R>, V, RegistryIdMap<R, V>, HashMap<V, RegistryId<R>>>,
        );
        self
    }

    fn keep_changable_one_sided_data<R, V>(&mut self) -> &mut Self
    where
        R: Registry,
        V: Sync + Send + 'static,
    {
        csd_setup(self);
        self.world
            .resource_mut::<ConvertSystemsData>()
            .cancel(convert_system::<RegistryId<R>, V, RegistryIdMap<R, V>, ()>);
        self
    }
}
