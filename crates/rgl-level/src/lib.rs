use std::any::TypeId;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use fastrand::Rng;

/// Handles generation of layers:
/// - Each entity with [`Level`] component, that was recently created, will be used to generate Tilemaps using [`Layer`]
/// - These generated Tilemaps will be [`Children`] of this [`Level`]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, generate_layers);
    }
}

#[derive(Bundle, Default)]
pub struct LayerBundle {
    pub layer: Layer,
    pub layer_rng: LayerRng,
    pub layer_scratch: LayerScratch,
}

impl LayerBundle {
    pub fn from_layer(layer: Layer) -> Self {
        Self {
            layer,
            ..Default::default()
        }
    }
}

fn generate_layers(
    levels: Query<(&Level, Entity), Added<Level>>,
    mut layers: Query<(&Layer, &mut LayerRng, &mut LayerScratch)>,
    commands: ParallelCommands,
) {
    layers
        .par_iter_mut()
        .for_each(|(layer, mut layer_rng, mut layer_scratch)| {
            commands.command_scope(|mut commands| {
                for (level, level_entity) in levels.iter() {
                    layer.spawn(
                        &mut commands,
                        level,
                        &mut layer_rng.0,
                        &mut layer_scratch.0,
                        Vec2::ZERO,
                        level_entity,
                    );
                }
            });
        })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelObjectRarity(pub u32);

impl LevelObjectRarity {
    pub const COMMON: LevelObjectRarity = LevelObjectRarity(10);
    pub const RARE: LevelObjectRarity = LevelObjectRarity(5);
    pub const VERY_RARE: LevelObjectRarity = LevelObjectRarity(1);
}

impl Default for LevelObjectRarity {
    fn default() -> Self {
        Self::COMMON
    }
}

#[derive(Component, Default)]
pub struct Layer {
    /// Empty vector means, that the layer should be created for each level kind
    pub level_kinds: Vec<TypeId>,
    pub z_index: f32,
    pub tile_size: TilemapTileSize,
    pub grid_size: TilemapGridSize,
    pub texture: TilemapTexture,
    objects: Vec<(Box<dyn LevelObjectDyn>, LevelObjectRarity)>,
}

impl Layer {
    pub fn add_object<T: LevelObject>(&mut self, obj: Box<T>, rarity: LevelObjectRarity) {
        self.objects.push((obj, rarity));
    }
}

#[derive(Component, Default)]
pub struct LayerRng(pub Rng);

#[derive(Component, Default)]
pub struct LayerScratch(Vec<(usize, Vec<IVec2>)>);

type DefaultTileBundle = TileBundle;

impl Layer {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        level: &Level,
        rng: &mut Rng,
        scratch: &mut Vec<(usize, Vec<IVec2>)>,
        pos: Vec2,
        parent: Entity,
    ) {
        if !self.level_kinds.is_empty()
            && self
                .level_kinds
                .iter()
                .find(|v| level.kind.eq(*v))
                .is_none()
        {
            return;
        }

        let tile_level_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(level.size.as_uvec2().into());

        let mut any_tile = false;

        for (pos, _tile) in level.iter() {
            let mut scratch_i = 0usize;
            let mut rarity_sum = 0;
            for (i, (object, object_rarity)) in self.objects.iter().enumerate() {
                if scratch.len() == scratch_i {
                    scratch.push(Default::default());
                }
                let (object_i, c_scratch) = &mut scratch[scratch_i];
                c_scratch.clear();
                if object.check(level, pos, c_scratch) {
                    *object_i = i;
                    rarity_sum += object_rarity.0;
                    scratch_i += 1;
                }
            }

            if scratch_i != 0 {
                any_tile = true;
                let mut chosen_object = rng.u32(0..rarity_sum);

                let (object_i, object_scratch) = scratch
                    .iter_mut()
                    .find(|(index, _)| {
                        let object_rarity = self.objects[*index].1 .0;
                        if object_rarity >= chosen_object {
                            true
                        } else {
                            chosen_object -= object_rarity;
                            false
                        }
                    })
                    .unwrap();

                self.objects[*object_i].0.spawn(
                    &object_scratch,
                    commands,
                    TileBundle {
                        tilemap_id: TilemapId(tile_level_entity),
                        ..Default::default()
                    },
                    tile_level_entity,
                    &mut tile_storage,
                );
            }
        }

        if any_tile {
            commands
                .entity(tile_level_entity)
                .insert(TilemapBundle {
                    tile_size: self.tile_size,
                    grid_size: self.grid_size,
                    texture: self.texture.clone(),
                    map_type: TilemapType::Square,
                    size: level.size.as_uvec2().into(),
                    storage: tile_storage,
                    transform: Transform::from_translation(Vec3::new(pos.x, pos.y, self.z_index)),
                    ..Default::default()
                })
                .set_parent(parent);
        } else {
            commands.entity(tile_level_entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct Level {
    pub tiles: Vec<TypeId>,
    pub kind: TypeId,
    pub size: IVec2,
}

pub struct JustLevel;

impl Default for Level {
    fn default() -> Self {
        Self {
            tiles: vec![],
            kind: TypeId::of::<JustLevel>(),
            size: IVec2::ZERO,
        }
    }
}

#[derive(Bundle, Default)]
pub struct LevelBundle {
    pub transform: TransformBundle,
    pub level: Level,
}

impl Level {
    pub fn iter(&self) -> impl Iterator<Item = (IVec2, TypeId)> + '_ {
        self.tiles.iter().cloned().enumerate().map(|(index, tile)| {
            (
                IVec2::new(index as i32 % self.size.x, index as i32 / self.size.y),
                tile,
            )
        })
    }

    pub fn get(&self, pos: IVec2) -> Option<TypeId> {
        if pos.x >= self.size.x || pos.x <= -1 || pos.y >= self.size.y || pos.y <= -1 {
            None
        } else {
            Some(self.tiles[(pos.x + pos.y * self.size.x) as usize])
        }
    }
}

pub trait LevelObject: Sync + Send + 'static {
    type TileBundle: Bundle;

    fn bundle(&self, index: usize) -> Self::TileBundle;

    fn check(&self, level: &Level, pos: IVec2, fill: &mut Vec<IVec2>) -> bool;
}

pub struct DefaultLevelObject<B> {
    pub level_kind: Option<TypeId>,
    pub tiles: [Option<TypeId>; 9],
    pub bundle: B,
}

impl<B> LevelObject for DefaultLevelObject<B>
where
    B: Bundle,
    B: Clone,
    B: Sync + Send,
    B: 'static,
{
    type TileBundle = B;

    fn bundle(&self, _index: usize) -> Self::TileBundle {
        // DefaultLevelObject is just one tile, so the index should always be 0
        debug_assert!(_index == 0);
        self.bundle.clone()
    }

    fn check(&self, level: &Level, pos: IVec2, fill: &mut Vec<IVec2>) -> bool {
        if matches!(self.level_kind, Some(level_kind) if level_kind == level.kind) {
            return false;
        }
        let mut i = 0;
        for dy in -1..2 {
            for dx in -1..2 {
                if let Some(level_tile) = self.tiles[i] {
                    let c_pos = pos + IVec2::new(dx, dy);
                    if level.get(c_pos) != Some(level_tile) {
                        return false;
                    }
                }
                i += 1;
            }
        }
        fill.push(pos);
        true
    }
}

trait LevelObjectDyn: Sync + Send + 'static {
    fn spawn(
        &self,
        positions: &Vec<IVec2>,
        commands: &mut Commands,
        tile_bundle: DefaultTileBundle,
        parent: Entity,
        tile_storage: &mut TileStorage,
    );

    fn check(&self, level: &Level, pos: IVec2, fill: &mut Vec<IVec2>) -> bool;
}

impl<T: LevelObject> LevelObjectDyn for T {
    fn spawn(
        &self,
        positions: &Vec<IVec2>,
        commands: &mut Commands,
        tile_bundle: DefaultTileBundle,
        parent: Entity,
        tile_storage: &mut TileStorage,
    ) {
        for (index, pos) in positions.into_iter().enumerate() {
            let pos = TilePos::from(pos.as_uvec2());
            tile_storage.set(
                &pos,
                commands
                    .spawn(tile_bundle.clone())
                    .insert((self.bundle(index), pos))
                    .set_parent(parent)
                    .id(),
            );
        }
    }

    fn check(&self, level: &Level, pos: IVec2, fill: &mut Vec<IVec2>) -> bool {
        T::check(&self, level, pos, fill)
    }
}
