use bevy::{prelude::*, ui::FocusPolicy};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, play_button);
    }
}

pub fn play_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(256.0),
                    height: Val::Px(96.0),
                    ..Default::default()
                },
                image: UiImage::new(asset_server.load("buttons/menu/generic_button_normal.png")),
                ..Default::default()
            });
        });
}
