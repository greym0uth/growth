use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 4.0, y: 4.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 128, y: 512 };
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 4,
    y: CHUNK_SIZE.y,
};

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) {
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
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
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
            if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                println!("Spawning ({x}, {y})");
                chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                spawn_chunk(&mut commands, &asset_server, IVec2::new(x, y));
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform, &TileStorage)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform, storage) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.x;
            let distance = (camera_transform.translation.x - chunk_pos).abs();
            if distance > CHUNK_SIZE.x as f32 * TILE_SIZE.x * 3. {
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
            }
        }
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Growth"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: RENDER_CHUNK_SIZE,
            ..Default::default()
        })
        .add_plugin(TilemapPlugin)
        .insert_resource(ChunkManager::default())
        .add_startup_system(startup)
        .add_system(spawn_chunks_around_camera)
        .add_system(despawn_outofrange_chunks)
        .run();
}
