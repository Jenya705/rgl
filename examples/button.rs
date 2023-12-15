use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport};
use rgl_ui::UiCommandsExt;

#[derive(Component)]
pub struct RglButton;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), PixelViewport));
    commands.make_button(
        "Hello!",
        Style {
            ..Default::default()
        },
        RglButton,
        asset_server,
    );
}
