use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rgl_level::{
    DefaultLevel, DefaultLevelObject, Layer, LayerBundle, Level, LevelBundle, LevelObjectRarity,
    LevelPlugin,
};
use rgl_registry::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(LevelPlugin)
        .register_two_sided_data_id2value::<LevelTileRegistry, &'static str>("level_tile")
        .add_systems(Startup, setup)
        .run()
}

new_registry!(LevelTileRegistry, u16);

new_registry_items!(LevelTileRegistry { Floor, Wall, Air });

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut layer = Layer::default();
    layer.grid_size = Vec2::splat(16.0).into();
    layer.tile_size = Vec2::splat(16.0).into();
    layer.texture = TilemapTexture::Single(asset_server.load("test_tiles.png"));

    layer.add_object(
        Box::new(DefaultLevelObject::new(
            Some(RegistryId::new::<DefaultLevel>()),
            {
                let mut tiles = [None; 9];
                tiles[4] = Some(RegistryId::new::<Floor>());
                tiles
            },
            TileTextureIndex(0),
        )),
        LevelObjectRarity::COMMON,
    );

    layer.add_object(
        Box::new(DefaultLevelObject::new(
            Some(RegistryId::new::<DefaultLevel>()),
            {
                let mut tiles = [None; 9];
                tiles[4] = Some(RegistryId::new::<Wall>());
                tiles
            },
            TileTextureIndex(1),
        )),
        LevelObjectRarity::COMMON,
    );

    commands.spawn(LayerBundle::from_layer(layer));

    commands.spawn(LevelBundle::from_level(Level::from_tiles([
        [
            RegistryId::new::<Floor>(),
            RegistryId::new::<Floor>(),
        ],
        [
            RegistryId::new::<Air>(),
            RegistryId::new::<Wall>(),
        ],
    ])));
}
