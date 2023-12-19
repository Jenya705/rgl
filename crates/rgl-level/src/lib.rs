use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use fastrand::Rng;
use rgl_registry::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_two_sided_data_id2value::<LevelKindRegistry, &'static str>("level_kind")
            .register_two_sided_data_id2value::<DefaultLevel, &'static str>("default");
    }
}

pub struct LayerPlugin<R: Registry>(PhantomData<R>);

impl<R: Registry> Plugin for LayerPlugin<R> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, generate_layers::<R>);
    }
}

impl<R: Registry> Default for LayerPlugin<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

new_registry!(LevelKindRegistry, u16);

#[derive(Bundle)]
pub struct LayerBundle<R: Registry> {
    pub layer: Layer<R>,
    pub layer_rng: LayerRng,
    pub layer_scratch: LayerScratch,
}

impl<R: Registry> LayerBundle<R> {
    pub fn from_layer(layer: Layer<R>) -> Self {
        Self {
            layer,
            layer_rng: LayerRng::default(),
            layer_scratch: LayerScratch::default(),
        }
    }
}

fn generate_layers<R: Registry>(
    levels: Query<(&Level<R>, Entity), Added<Level<R>>>,
    mut layers: Query<(&Layer<R>, &mut LayerRng, &mut LayerScratch)>,
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

#[derive(Component)]
pub struct Layer<R: Registry> {
    /// Empty vector means, that the layer should be created for each level kind
    pub level_kinds: Vec<RegistryId<LevelKindRegistry>>,
    pub z_index: f32,
    pub tile_size: TilemapTileSize,
    pub grid_size: TilemapGridSize,
    pub texture: TilemapTexture,
    objects: Vec<(Box<dyn LevelObjectDyn<R>>, LevelObjectRarity)>,
}

impl<R: Registry> Layer<R> {
    pub fn add_object<T: LevelObject<R>>(&mut self, obj: Box<T>, rarity: LevelObjectRarity) {
        self.objects.push((obj, rarity));
    }
}

impl<R: Registry> Default for Layer<R> {
    fn default() -> Self {
        Self {
            level_kinds: Vec::default(),
            z_index: 0.0,
            tile_size: Default::default(),
            grid_size: Default::default(),
            texture: Default::default(),
            objects: Vec::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct LayerRng(pub Rng);

#[derive(Component, Default)]
pub struct LayerScratch(Vec<(usize, Vec<IVec2>)>);

type DefaultTileBundle = TileBundle;

impl<R: Registry> Layer<R> {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        level: &Level<R>,
        rng: &mut Rng,
        scratch: &mut Vec<(usize, Vec<IVec2>)>,
        pos: Vec2,
        parent: Entity,
    ) {
        if !self.level_kinds.is_empty()
            && self.level_kinds.iter().find(|v| level.kind.eq(v)).is_none()
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
pub struct Level<R: Registry> {
    pub tiles: Vec<RegistryId<R>>,
    pub kind: RegistryId<LevelKindRegistry>,
    pub size: IVec2,
}

impl<R: Registry> Level<R> {
    pub fn from_tiles<const COLUMNS: usize, const ROWS: usize>(
        tiles: [[RegistryId<R>; COLUMNS]; ROWS],
    ) -> Self {
        let mut tiles_vec = Vec::new();
        tiles_vec.reserve(COLUMNS * ROWS);
        // SAFETY: tiles_vec.reserve() ensures, that we will have space for C * R items in vec
        // tiles and tiles_vec can not overlap, because vec is allocated dynamically
        unsafe {
            std::ptr::copy_nonoverlapping(tiles[0].as_ptr(), tiles_vec.as_mut_ptr(), COLUMNS * ROWS)
        };
        // SAFETY: we copied C * R items in the last statement
        unsafe {
            tiles_vec.set_len(COLUMNS * ROWS);
        }

        Self {
            tiles: tiles_vec,
            kind: RegistryId::new::<DefaultLevel>(),
            size: IVec2::new(COLUMNS as i32, ROWS as i32),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (IVec2, RegistryId<R>)> + '_ {
        self.tiles.iter().cloned().enumerate().map(|(index, tile)| {
            (
                IVec2::new(index as i32 % self.size.x, index as i32 / self.size.y),
                tile,
            )
        })
    }

    pub fn get(&self, pos: IVec2) -> Option<RegistryId<R>> {
        if pos.x >= self.size.x || pos.x <= -1 || pos.y >= self.size.y || pos.y <= -1 {
            None
        } else {
            Some(self.tiles[(pos.x + pos.y * self.size.x) as usize].clone())
        }
    }
}

new_registry_items!(LevelKindRegistry { DefaultLevel });

#[derive(Bundle)]
pub struct LevelBundle<R: Registry> {
    pub transform: TransformBundle,
    pub level: Level<R>,
}

impl<R: Registry> LevelBundle<R> {
    pub fn from_level(level: Level<R>) -> Self {
        Self {
            level,
            transform: TransformBundle::default(),
        }
    }
}

pub trait LevelObject<R: Registry>: Sync + Send + 'static {
    type TileBundle: Bundle;

    fn bundle(&self, index: usize) -> Self::TileBundle;

    fn check(&self, level: &Level<R>, pos: IVec2, fill: &mut Vec<IVec2>) -> bool;
}

pub struct DefaultLevelObject<R: Registry, B> {
    pub level_kind: Option<RegistryId<LevelKindRegistry>>,
    pub tiles: [Option<RegistryId<R>>; 9],
    pub bundle: B,
}

impl<R: Registry, B> DefaultLevelObject<R, B> {
    pub fn new(
        level_kind: Option<RegistryId<LevelKindRegistry>>,
        tiles: [Option<RegistryId<R>>; 9],
        bundle: B,
    ) -> Self {
        Self {
            level_kind,
            tiles,
            bundle,
        }
    }
}

impl<R: Registry, B> LevelObject<R> for DefaultLevelObject<R, B>
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

    fn check(&self, level: &Level<R>, pos: IVec2, fill: &mut Vec<IVec2>) -> bool {
        if matches!(&self.level_kind, Some(level_kind) if level.kind.ne(level_kind)) {
            return false;
        }
        let mut i = 0;
        for dy in -1..2 {
            for dx in -1..2 {
                if let Some(rid) = &self.tiles[i] {
                    let c_pos = pos + IVec2::new(dx, dy);
                    if !matches!(level.get(c_pos), Some(l_tile) if l_tile.eq(rid)) {
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

trait LevelObjectDyn<R: Registry>: Sync + Send + 'static {
    fn spawn(
        &self,
        positions: &Vec<IVec2>,
        commands: &mut Commands,
        tile_bundle: DefaultTileBundle,
        parent: Entity,
        tile_storage: &mut TileStorage,
    );

    fn check(&self, level: &Level<R>, pos: IVec2, fill: &mut Vec<IVec2>) -> bool;
}

impl<R: Registry, T: LevelObject<R>> LevelObjectDyn<R> for T {
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

    fn check(&self, level: &Level<R>, pos: IVec2, fill: &mut Vec<IVec2>) -> bool {
        T::check(&self, level, pos, fill)
    }
}
