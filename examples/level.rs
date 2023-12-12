use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rgl_level::{
    DefaultLevelObject, DefaultLevel, Layer, LayerBundle, Level, LevelBundle, LevelObjectRarity,
    LevelPlugin,
};
use rgl_registry::{
    new_registry, new_registry_item, RegistryAppMethods, RegistryId, RegistryPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(RegistryPlugin::<LevelTileRegistry>::default())
        .register_item::<Floor>("floor")
        .register_item::<Wall>("wall")
        .register_item::<Air>("air")
        .register_item::<LevelTileRegistry>("level_tile")
        .add_systems(Startup, setup)
        .run()
}

new_registry!(LevelTileRegistry, u16);
new_registry_item!(Floor, LevelTileRegistry);
new_registry_item!(Wall, LevelTileRegistry);
new_registry_item!(Air, LevelTileRegistry);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut layer = Layer::default();
    layer.grid_size = Vec2::splat(16.0).into();
    layer.tile_size = Vec2::splat(16.0).into();
    layer.texture = TilemapTexture::Single(asset_server.load("test_tiles.png"));

    layer.add_object(
        Box::new(DefaultLevelObject::new(
            Some(RegistryId::from_item::<DefaultLevel>()),
            {
                let mut tiles = [None; 9];
                tiles[4] = Some(RegistryId::from_item::<Floor>());
                tiles
            },
            TileTextureIndex(0),
        )),
        LevelObjectRarity::COMMON,
    );

    layer.add_object(
        Box::new(DefaultLevelObject::new(
            Some(RegistryId::from_item::<DefaultLevel>()),
            {
                let mut tiles = [None; 9];
                tiles[4] = Some(RegistryId::from_item::<Wall>());
                tiles
            },
            TileTextureIndex(1),
        )),
        LevelObjectRarity::COMMON,
    );

    commands.spawn(LayerBundle::from_layer(layer));

    commands.spawn(LevelBundle::from_level(Level::from_tiles([
        [
            RegistryId::from_item::<Floor>(),
            RegistryId::from_item::<Floor>(),
        ],
        [
            RegistryId::from_item::<Air>(),
            RegistryId::from_item::<Wall>(),
        ],
    ])));
}
