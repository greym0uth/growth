use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;
use chunk_managment::ChunkManagmentPlugin;
mod chunk_managment;

const CHUNK_SIZE: UVec2 = UVec2 { x: 128, y: 512 };
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 4,
    y: CHUNK_SIZE.y,
};

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
        .add_plugin(ChunkManagmentPlugin)
        .add_startup_system(startup)
        .run();
}
