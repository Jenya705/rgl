use bevy::{ecs::system::EntityCommands, prelude::*, transform::commands};

pub trait UiCommandsExt<'w, 's, T: Component> {
    fn make_button(
        &mut self,
        text: &str,
        style: Style,
        marker: T,
        asset_server: Res<'w, AssetServer>,
    ) -> EntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: Component> UiCommandsExt<'w, 's, T> for Commands<'w, 's> {
    fn make_button(
        &mut self,
        text: &str,
        style: Style,
        marker: T,
        asset_server: Res<'w, AssetServer>,
    ) -> EntityCommands<'w, 's, '_> {
        let mut ec = self.spawn((
            ButtonBundle {
                image: UiImage::new(asset_server.get_handle("buttons/menu/"))..Default::default(),
            },
            marker,
        ));

        ec
    }
}
