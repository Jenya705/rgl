use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PixelViewport));
}
