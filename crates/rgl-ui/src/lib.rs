use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait UiCommandsExt<'w, 's, T: Component> {
    fn make_button(
        &mut self,
        text: impl Into<String>,
        style: Style,
        marker: T,
        asset_server: Res<'w, AssetServer>,
    ) -> EntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: Component> UiCommandsExt<'w, 's, T> for Commands<'w, 's> {
    fn make_button(
        &mut self,
        text: impl Into<String>,
        style: Style,
        marker: T,
        asset_server: Res<'w, AssetServer>,
    ) -> EntityCommands<'w, 's, '_> {
        let mut ec = self.spawn((
            ButtonBundle {
                image: UiImage::new(
                    asset_server
                        .get_handle("buttons/menu/generic_button_normal.png")
                        .unwrap(),
                ),
                style,
                ..Default::default()
            },
            marker,
        ));

        ec.with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(text, TextStyle::default()),
                ..Default::default()
            });
        });

        ec
    }
}
