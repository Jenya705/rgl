use bevy::{ecs::system::EntityCommands, prelude::*};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenericButtonHandles>()
            .add_systems(PostUpdate, button_system)
            .add_systems(Startup, load_generic_button_handles);
    }
}

#[derive(Resource, Default)]
struct GenericButtonHandles {
    pressed: Handle<Image>,
    hovered: Handle<Image>,
    normal: Handle<Image>,
}

fn load_generic_button_handles(
    asset_server: Res<AssetServer>,
    mut handles: ResMut<GenericButtonHandles>,
) {
    handles.pressed = asset_server.load("buttons/menu/generic_button_pressed.png");
    handles.hovered = asset_server.load("buttons/menu/generic_button_hover.png");
    handles.normal = asset_server.load("buttons/menu/generic_button_normal.png");
}

fn button_system(
    mut query: Query<(&Interaction, &mut UiImage, &UiElement), Changed<Interaction>>,
    handles: Res<GenericButtonHandles>,
) {
    for (interaction, mut image, element) in &mut query {
        if !matches!(element, UiElement::Button) {
            continue;
        };

        match interaction {
            Interaction::Pressed => *image = UiImage::new(handles.pressed.clone()),
            Interaction::Hovered => *image = UiImage::new(handles.hovered.clone()),
            Interaction::None => *image = UiImage::new(handles.normal.clone()),
        }
    }
}

#[derive(Component)]
enum UiElement {
    Button,
}

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

/// This trait provides extension methods for spawning different UI presets.
pub trait UiCommandsExt<'w, 's> {
    /// Spawns a green button with the specified label.
    /// A marker component may be attached to the button.
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
                style,
                ..Default::default()
            },
            marker,
            UiElement::Button,
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
