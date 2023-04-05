use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::chunk_managment::{chunk_pos_to_world_pos, ChunkManager, ChunkPos};

pub trait TileData {
    fn moisture(&self) -> u8;
    fn max_moisture(&self) -> u8;
    fn nutrients(&self) -> u8;
    fn max_nutrients(&self) -> u8;
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tile<T> {
    data: T,
}

pub trait TileBuilder {
    type TileDataDesc: TileData;

    fn create_tile() -> Tile<Self::TileDataDesc>;
}

pub trait TileSimulate<T: Event> {
    type TileFilter: Component;

    fn simulate(
        chunk_manager: Res<ChunkManager>,
        chunks: Query<&TileStorage>,
        tiles: Query<(Entity, &TilePos, &ChunkPos, &Self::TileFilter)>,
        events: EventWriter<TileEvent>,
    );
}

pub struct TileModifierDesc {
    pub moisture: u8,
    pub nutrients: u8,
}

pub enum TileEvent {
    Add(Entity, TileModifierDesc),
    Subtract(Entity, TileModifierDesc),
}

pub struct Air;

#[derive(Default, Clone, Copy)]
pub struct Dirt {
    moisture: u8,
    max_moisture: u8,
    nutrients: u8,
    max_nutrients: u8,
}

impl TileData for Dirt {
    fn moisture(&self) -> u8 {
        self.moisture
    }

    fn max_moisture(&self) -> u8 {
        self.max_moisture
    }

    fn nutrients(&self) -> u8 {
        self.nutrients
    }

    fn max_nutrients(&self) -> u8 {
        self.max_nutrients
    }
}

impl TileBuilder for Tile<Dirt> {
    type TileDataDesc = Dirt;

    fn create_tile() -> Tile<Self::TileDataDesc> {
        let data = Dirt {
            moisture: 0,
            max_moisture: (u8::MAX as f32 / 0.75) as u8,
            nutrients: 0,
            max_nutrients: (u8::MAX as f32 * 0.5) as u8,
        };
        Tile::<Dirt> { data }
    }
}

const WORLD_SQUARE_NEIGHBOR_POSITONS: [IVec2; 8] = [
    IVec2 { x: -1, y: -1 },
    IVec2 { x: -1, y: 0 },
    IVec2 { x: -1, y: 1 },
    IVec2 { x: 0, y: -1 },
    IVec2 { x: 0, y: 1 },
    IVec2 { x: 1, y: -1 },
    IVec2 { x: 1, y: 0 },
    IVec2 { x: 1, y: 1 },
];

pub struct WorldNeighbors {
    neighbors: Vec<IVec2>,
}

impl WorldNeighbors {
    pub fn get_square_neighboring_positions(position: IVec2) -> WorldNeighbors {
        let neighbors: Vec<IVec2> = WORLD_SQUARE_NEIGHBOR_POSITONS
            .into_iter()
            .map(|direction| position + direction)
            .collect();
        WorldNeighbors { neighbors }
    }

    pub fn iter(&self) -> impl Iterator<Item = IVec2> {
        self.neighbors.clone().into_iter()
    }
}

impl Dirt {
    fn simulate(
        chunk_manager: Res<ChunkManager>,
        chunks: Query<&TileStorage>,
        tiles: Query<(Entity, &TilePos, &ChunkPos, &Tile<Dirt>)>,
        mut events: EventWriter<TileEvent>,
    ) {
        for (entity, position, chunk_pos, _tile) in tiles.iter() {
            let world_pos = chunk_pos_to_world_pos(chunk_pos, position);
            for neighbor_pos in WorldNeighbors::get_square_neighboring_positions(world_pos).iter() {
                if let Some(neighbor) = chunk_manager.get(&neighbor_pos, &chunks) {
                    if tiles.get(neighbor).is_ok() {
                        events.send(TileEvent::Add(
                            neighbor,
                            TileModifierDesc {
                                moisture: 1,
                                nutrients: 1,
                            },
                        ));
                        events.send(TileEvent::Subtract(
                            entity,
                            TileModifierDesc {
                                moisture: 1,
                                nutrients: 1,
                            },
                        ));
                    }
                }
            }
        }
    }
}

pub enum Tree {
    Root,
    Trunk,
    Leaf,
}

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        let _ = app.add_system(Dirt::simulate);
    }
}
