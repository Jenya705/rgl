use bevy::prelude::*;
use rgl_input::{BindingPlugin, BindingApp, Key, Binding, BindingId};

pub struct YoBinding;

pub struct NotYoBinding;

fn main() {
    App::new()
        .add_plugins(BindingPlugin)
        .add_plugins(DefaultPlugins)
        .register_binding::<YoBinding>(Key::Keyboard(KeyCode::Q))
        .register_binding::<NotYoBinding>(Key::Keyboard(KeyCode::E))
        .add_systems(Update, yo_not_yo)
        .run();
}

fn yo_not_yo(yo_binding: Binding<YoBinding>, not_yo_binding: Binding<NotYoBinding>, bindings: Res<Input<BindingId>>) {
    if bindings.just_pressed(yo_binding.id) {
        println!("Yo!");
    } else if bindings.just_pressed(not_yo_binding.id) {
        println!("Not yo :(");
    }
}
