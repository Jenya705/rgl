use bevy::{prelude::*, render::{RenderPlugin, settings::{RenderCreation, WgpuSettings, Backends}}};
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport};
use rgl_ui::{UiCommandsExt, UiPlugin};

#[derive(Component)]
pub struct RglButton;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..Default::default()
                    }),
                }),
        )
        .add_plugins((PixelCameraPlugin, UiPlugin))
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
            left: Val::Percent(20.0),
            top: Val::Percent(20.0),
            ..Default::default()
        },
        24.0,
        RglButton,
        asset_server,
    );
}
