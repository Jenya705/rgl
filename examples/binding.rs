use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use rgl_input::*;
use rgl_registry::new_registry_items;

fn main() {
    App::new()
        .add_plugins(BindingPlugin)
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..Default::default()
            }),
        }))
        .add_systems(Update, yo_not_yo)
        .add_systems(Startup, setup)
        .register_binding::<YoBinding>("yo", Key::Keyboard(KeyCode::Q))
        .register_binding::<NotYoBinding>("not_yo", Key::Keyboard(KeyCode::E))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
}

new_registry_items!(BindingRegistry {
    YoBinding,
    NotYoBinding,
});

fn yo_not_yo(bindings: Res<BindingStates>, mut sprite: Query<&mut Sprite>) {
    let mut sprite = sprite.single_mut();

    if bindings.value_ty::<YoBinding>().unwrap().just_pressed() {
        sprite.color = Color::YELLOW;
    } else if bindings.value_ty::<NotYoBinding>().unwrap().just_pressed() {
        sprite.color = Color::BLUE;
    }
}
