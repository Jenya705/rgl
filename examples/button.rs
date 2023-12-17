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
            width: Val::Px(128.0),
            height: Val::Px(48.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            ..Default::default()
        },
        24.0,
        RglButton,
        asset_server,
    );
}
