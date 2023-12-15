use bevy::{ecs::system::EntityCommands, prelude::*};

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

pub trait UiCommandsExt<'w, 's, T: Component> {
    fn make_button(
        &mut self,
        text: impl Into<String>,
        style: Style,
        marker: T,
        asset_server: Res<AssetServer>,
    ) -> EntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: Spawn<'w, 's>, C: Component> UiCommandsExt<'w, 's, C> for T {
    fn make_button(
        &mut self,
        text: impl Into<String>,
        style: Style,
        marker: C,
        asset_server: Res<AssetServer>,
    ) -> EntityCommands<'w, 's, '_> {
        let mut ec = self.spawn((
            ButtonBundle {
                image: UiImage::new(asset_server.load("buttons/menu/generic_button_normal.png")),
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
