use std::{
    any::{type_name, Any},
    marker::PhantomData,
};

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, InputSystem},
    prelude::*,
    utils::HashMap,
};

pub struct BindingPlugin;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingSet;

impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<BindingId>>()
            .init_resource::<Bindings>()
            .configure_sets(PreUpdate, BindingSet.after(InputSystem))
            .add_systems(PreUpdate, key_binding_input.in_set(BindingSet));
    }
}

pub trait BindingApp {
    fn register_binding<T: Any>(&mut self, default_key: Key) -> &mut App;
}

impl BindingApp for App {
    fn register_binding<T: Any>(&mut self, default_key: Key) -> &mut App {
        let _ = self
            .world
            .resource_mut::<Bindings>()
            .add_binding(BindingData::new(type_name::<T>().into(), default_key));
        self
    }   
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Key {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindingId(usize);

pub struct BindingData {
    pub name: String,
    pub default_key: Key,
    pub set_key: Option<Key>,
}

impl BindingData {
    pub fn new(name: String, default_key: Key) -> Self {
        Self {
            name,
            default_key,
            set_key: Some(default_key),
        }
    }
}

/// About saveability of this resource:
/// - Categories should not be saved, because it can not be changed by the player
/// - BindingId is not a thing for saves, because we can not assure,
///   that on all version the order of binding's register will be the same, so that save should assure,
///   that binding's id are their names, which won't be changed
/// - Default keys are also not a thing
/// 
/// Save should look like this:
/// ```json
/// {
///     "binding_name": "set_key"
/// }
/// ```
#[derive(Resource, Default)]
pub struct Bindings {
    categories: HashMap<String, Vec<BindingId>>,
    bindings: Vec<BindingData>,
    keys: HashMap<Key, BindingId>,
}

impl Bindings {
    pub(crate) fn add_binding(&mut self, binding: BindingData) -> BindingId {
        let bid = BindingId(self.bindings.len());
        self.keys.insert(binding.set_key.unwrap(), bid);
        self.bindings.push(binding);
        bid
    }

    /// # Panics
    /// When binding is already added to the category
    pub(crate) fn add_to_category(&mut self, bid: BindingId, category: &str) {
        if !self.categories.contains_key(category) {
            self.categories.insert(category.into(), vec![bid]);
        } else {
            let category = self.categories.get_mut(category).unwrap();
            debug_assert!(category.iter().find(|bid2| bid.eq(bid2)).is_none());
            category.push(bid);
        }
    }

    pub(crate) fn change_binding_key(&mut self, bid: BindingId, new_key: Key) -> Option<BindingId> {
        let cur_bid = self.keys.get(&new_key).cloned();
        if let Some(cur_bid) = cur_bid {
            if bid == cur_bid {
                return None;
            } else {
                self.bindings[cur_bid.0].set_key = None;
            }
        }
        let binding = &mut self.bindings[bid.0];
        if let Some(set_key) = binding.set_key {
            self.keys.remove(&set_key);
        }
        binding.set_key = Some(new_key);
        self.keys.insert(new_key, bid);
        cur_bid
    }

    pub(crate) fn find_binding(&self, name: &str) -> Option<BindingId> {
        self.bindings
            .iter()
            .enumerate()
            .find(|(_, bdata)| bdata.name == name)
            .map(|(bid, _)| BindingId(bid))
    }
}

pub type Binding<'s, T> = Local<'s, BindingIdState<T>>;

#[doc(hidden)]
pub struct BindingIdState<T: Any> {
    pub id: BindingId,
    _marker: PhantomData<T>,
}

impl<T: Any> FromWorld for BindingIdState<T> {
    fn from_world(world: &mut World) -> Self {
        BindingIdState {
            id: world
                .resource::<Bindings>()
                .find_binding(type_name::<T>())
                .expect(
                "Systems with bindings should be initialised after the required binding is added",
            ),
            _marker: PhantomData,
        }
    }
}

fn key_binding_input(
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    bindings: Res<Bindings>,
    mut binding: ResMut<Input<BindingId>>,
) {
    binding.clear();
    for (key, pressed) in keyboard
        .read()
        .filter_map(|v| {
            v.key_code
                .map(|kc| (Key::Keyboard(kc), v.state.is_pressed()))
        })
        .chain(
            mouse
                .read()
                .map(|v| (Key::Mouse(v.button), v.state.is_pressed())),
        )
    {
        if let Some(bid) = bindings.keys.get(&key) {
            if pressed {
                binding.press(*bid)
            } else {
                binding.release(*bid)
            }
        }
    }
}