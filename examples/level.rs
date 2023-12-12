use std::any::TypeId;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rgl_level::{DefaultLevelObject, Layer, LayerBundle, Level, LevelBundle, LevelPlugin, LevelObjectRarity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(LevelPlugin)
        .add_systems(Startup, setup)
        .run()
}

struct Wall;

struct Floor;

struct Air;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut layer = Layer::default();
    layer.grid_size = Vec2::splat(16.0).into();
    layer.tile_size = Vec2::splat(16.0).into();
    layer.texture = TilemapTexture::Single(asset_server.load("test_tiles.png"));

    layer.add_object(Box::new(DefaultLevelObject {
        bundle: TileTextureIndex(0),
        level_kind: None,
        tiles: [
            None,
            None,
            None,
            None,
            Some(TypeId::of::<Floor>()),
            None,
            None,
            None,
            None,
        ],
    }), LevelObjectRarity::COMMON);

    layer.add_object(Box::new(DefaultLevelObject {
        bundle: TileTextureIndex(1),
        level_kind: None,
        tiles: [
            None,
            None,
            None,
            None,
            Some(TypeId::of::<Wall>()),
            None,
            None,
            None,
            None,
        ],
    }), LevelObjectRarity::COMMON);

    commands.spawn(LayerBundle::from_layer(layer));

    commands.spawn(LevelBundle {
        level: Level {
            tiles: vec![
                TypeId::of::<Floor>(),
                TypeId::of::<Floor>(),
                TypeId::of::<Wall>(),
                TypeId::of::<Air>(),
            ],
            size: IVec2::splat(2),
            ..Default::default()
        },
        ..Default::default()
    });
}
