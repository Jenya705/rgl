use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, InputSystem},
    prelude::*,
};
use rgl_registry::{
    new_registry, RegistryAppExt, RegistryItem, RegistryOneSidedDataCell,
    RegistryTwoSidedDataCellId2Value,
};

pub struct BindingPlugin;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingSet;

impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, BindingSet.after(InputSystem))
            .add_systems(PreUpdate, key_binding_input.in_set(BindingSet))
            .register_two_sided_data_id2value::<BindingRegistry, &'static str>("bindings")
            .register_two_sided_data_id2value::<BindingCategoryRegistry, &'static str>("binding_categories");
    }
}

new_registry!(BindingRegistry, u16);

new_registry!(BindingCategoryRegistry, u8);

pub trait BindingAppExt {
    fn register_binding<I>(&mut self, name: &'static str, default_key: Key) -> &mut Self
    where
        I: RegistryItem<Registry = BindingRegistry>;
}

impl BindingAppExt for App {
    fn register_binding<I>(&mut self, name: &'static str, default_key: Key) -> &mut Self
    where
        I: RegistryItem<Registry = BindingRegistry>,
    {
        self.register_two_sided_data_id2value::<I, &'static str>(name)
            .register_one_sided_data::<I, DefaultBindingKey>(DefaultBindingKey(default_key))
            .register_two_sided_data_id2value::<I, SetBindingKey>(SetBindingKey(default_key))
            .register_one_sided_data::<I, BindingState>(BindingState::default());
        self
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Key {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

pub struct DefaultBindingKey(pub Key);

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetBindingKey(pub Key);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingState(u8);

impl BindingState {
    const JUST_PRESSED: u8 = 0x1;
    const JUST_RELEASED: u8 = 0x2;
    const PRESSED: u8 = 0x4;

    pub fn just_pressed(&self) -> bool {
        self.0 & Self::JUST_PRESSED != 0
    }

    pub fn just_released(&self) -> bool {
        self.0 & Self::JUST_RELEASED != 0
    }

    pub fn pressed(&self) -> bool {
        self.0 & Self::PRESSED != 0
    }

    pub(crate) fn press(&mut self) {
        if !self.pressed() {
            self.0 = Self::JUST_PRESSED | Self::PRESSED;
        }
    }

    pub(crate) fn release(&mut self) {
        self.0 = Self::JUST_RELEASED;
    }

    pub(crate) fn clear(&mut self) {
        self.0 &= Self::PRESSED;
    }
}

pub type BindingStates = RegistryOneSidedDataCell<BindingRegistry, BindingState>;

fn key_binding_input(
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    binding_keys: Res<RegistryTwoSidedDataCellId2Value<BindingRegistry, SetBindingKey>>,
    mut binding_states: ResMut<BindingStates>,
) {
    binding_states.c1.iter_mut().for_each(BindingState::clear);
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
        if let Some(bid) = binding_keys.id(&SetBindingKey(key)) {
            let value = binding_states.value_mut(bid).unwrap();
            if pressed {
                value.press();
            } else {
                value.release();
            }
        }
    }
}
