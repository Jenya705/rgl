use bevy::{ecs::system::EntityCommands, prelude::*};

pub struct UiPlugin;

trait Spawn<'w, 's> {
    fn spawn<T: Bundle>(&mut self, bundle: T) -> EntityCommands<'w, 's, '_>;
}

impl<'w, 's> Spawn<'w, 's> for Commands<'w, 's> {
    fn spawn<T: Bundle>(&mut self, bundle: T) -> EntityCommands<'w, 's, '_> {
        self.spawn(bundle)
    }
}

impl<'w, 's> Spawn<'w, 's> for ChildBuilder<'w, 's, '_> {
    fn spawn<T: Bundle>(&mut self, bundle: T) -> EntityCommands<'w, 's, '_> {
        self.spawn(bundle)
    }
}

pub trait UiCommandsExt<'w, 's> {
    fn make_button<C: Component>(
        &mut self,
        text: impl Into<String>,
        style: Style,
        font_size: f32,
        marker: C,
        asset_server: Res<AssetServer>,
    ) -> EntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: Spawn<'w, 's>> UiCommandsExt<'w, 's> for T {
    fn make_button<C: Component>(
        &mut self,
        text: impl Into<String>,
        style: Style,
        font_size: f32,
        marker: C,
        asset_server: Res<AssetServer>,
    ) -> EntityCommands<'w, 's, '_> {
        let node_style = Style {
            width: style.width,
            height: style.height,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        };

        let mut ec = self.spawn((
            ButtonBundle {
                image: UiImage::new(asset_server.load("buttons/menu/generic_button_normal.png")),
                style,
                ..Default::default()
            },
            marker,
        ));

        ec.with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: node_style,
                    ..Default::default()
                })
                .with_children(|p| {
                    p.spawn(TextBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font: asset_server.load("fonts/manaspace.ttf"),
                                font_size,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                });
        });

        ec
    }
}
