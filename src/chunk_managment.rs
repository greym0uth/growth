use bevy::prelude::{App, Plugin};

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;

pub struct ChunkManagmentPlugin;

//TODO: add config resource.
const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 4.0, y: 4.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 128, y: 512 };
const CHUNK_SIM_RANGE: f32 = CHUNK_SIZE.x as f32 * TILE_SIZE.x * 3.;
const CHUNK_SPAWN_RANGE: f32 = CHUNK_SIM_RANGE + CHUNK_SIZE.x as f32;

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) -> Entity {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .insert(ChunkPos(chunk_pos))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
        0.0,
    ));
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TILE_SIZE.into(),
        size: CHUNK_SIZE.into(),
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TILE_SIZE,
        transform,
        ..Default::default()
    });

    tilemap_entity
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size = CHUNK_SIZE.as_ivec2();
    let tile_size = Vec2::from(TILE_SIZE).as_ivec2();
    camera_pos / (chunk_size * tile_size)
}

pub fn chunk_pos_to_world_pos(chunk_pos: &ChunkPos, tile_pos: &TilePos) -> IVec2 {
    let chunk_size = CHUNK_SIZE.as_ivec2();
    (chunk_pos.0 * chunk_size) + IVec2::new(tile_pos.x as i32, tile_pos.y as i32)
}

pub fn world_pos_to_chunk_pos(tile_pos: &IVec2) -> (ChunkPos, TilePos) {
    let chunk_size = CHUNK_SIZE.as_ivec2();
    let chunk_pos = *tile_pos / chunk_size;
    let tile_in_chunk_pos = *tile_pos - (chunk_pos * chunk_size);
    (
        ChunkPos(chunk_pos),
        TilePos::new(tile_in_chunk_pos.x as u32, tile_in_chunk_pos.y as u32),
    )
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.truncate());
        let y = 0;
        for x in (camera_chunk_pos.x - 2)..(camera_chunk_pos.x + 2) {
            if !chunk_manager.spawned_chunks.contains_key(&IVec2::new(x, y)) {
                println!("Spawning ({x}, {y})");
                let chunk_entity = spawn_chunk(&mut commands, &asset_server, IVec2::new(x, y));
                chunk_manager
                    .spawned_chunks
                    .insert(IVec2::new(x, y), chunk_entity);
            }
        }
    }
}

fn set_chunk_activity(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    asleep_chunks_query: Query<(Entity, &Transform, &TileStorage), With<ChunkAsleep>>,
    awake_chunks_query: Query<(Entity, &Transform, &TileStorage), Without<ChunkAsleep>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform, storage) in awake_chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.x;
            let distance = (camera_transform.translation.x - chunk_pos).abs();

            if distance > CHUNK_SIM_RANGE {
                commands.entity(entity).insert(ChunkAsleep);
            }
        }

        for (entity, chunk_transform, storage) in asleep_chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.x;
            let distance = (camera_transform.translation.x - chunk_pos).abs();
            if distance > CHUNK_SPAWN_RANGE {
                let x = (chunk_pos / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = 0;
                println!("Despawning ({x}, {y})");
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                for tile_option in storage.iter() {
                    if let Some(tile) = tile_option {
                        commands.entity(*tile).despawn();
                    }
                }
                commands.entity(entity).despawn();
            } else if distance < CHUNK_SIM_RANGE {
                commands.entity(entity).remove::<ChunkAsleep>();
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashMap<IVec2, Entity>,
}

impl ChunkManager {
    pub fn get(&self, world_position: &IVec2, chunks: &Query<&TileStorage>) -> Option<Entity> {
        let (chunk_pos, tile_in_chunk_pos) = world_pos_to_chunk_pos(world_position);
        if let Some(chunk) = self.spawned_chunks.get(&chunk_pos.0) {
            if let Ok(storage) = chunks.get(*chunk) {
                return storage.get(&tile_in_chunk_pos);
            }
        }
        None
    }
}

#[derive(Component)]
pub struct ChunkAsleep;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct ChunkPos(pub IVec2);

impl Plugin for ChunkManagmentPlugin {
    fn build(&self, app: &mut App) {
        let _ = app
            .add_system(spawn_chunks_around_camera)
            .add_system(set_chunk_activity)
            .insert_resource(ChunkManager::default());
    }
}
