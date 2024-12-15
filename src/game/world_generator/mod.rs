use std::sync::{Arc, Mutex, RwLock};

use bevy::{
    app::{App, Plugins, Startup, Update},
    prelude::{Commands, Component, Event, EventReader, EventWriter, Query, Res},
    tasks::AsyncComputeTaskPool,
};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use crate::data::world::{self, MemoryWorld, SimplePerlinGenerator, World, WorldGenerator};

pub mod vis;

/* -------------------------------------------------------------------------- */
/*                                   Plugin                                   */
/* -------------------------------------------------------------------------- */

pub struct WorldGeneratorPlugin {}

impl Default for WorldGeneratorPlugin {
    fn default() -> Self {
        WorldGeneratorPlugin {}
    }
}

impl bevy::prelude::Plugin for WorldGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GenerateWorldSignal>();
        app.add_systems(Startup, sys_setup);
        app.add_systems(Update, sys_update);
        app.add_systems(Update, sys_generate_chunk);
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Events                                   */
/* -------------------------------------------------------------------------- */

// TODO: There is a better way to do this without using an event
#[derive(Event, Debug, Clone)]
pub struct GenerateWorldSignal {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct ChunkUpdatedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct ChunkLoadedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct ChunkGeneratedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct ChunkDroppedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/* -------------------------------------------------------------------------- */

#[derive(Component)]
pub struct GameWorld {
    pub map: MemoryWorld,
    pub generator: SimplePerlinGenerator,
    pub prev_user_position: (f32, f32, f32),
}

impl GameWorld {
    pub fn new() -> GameWorld {
        let game_world = GameWorld {
            map: MemoryWorld::new(),
            generator: SimplePerlinGenerator::new(0),
            prev_user_position: (0.0, 0.0, 0.0),
        };

        game_world
    }
}

pub fn sys_setup(mut commands: Commands) {
    let game_world = GameWorld::new();
    commands.spawn(game_world);
}

pub fn sys_update(
    mut commands: Commands,
    camera: Query<(&bevy::prelude::Transform, &bevy::prelude::Camera3d)>,
    mut world: Query<&mut GameWorld>,
    mut ev_generate_world: EventWriter<GenerateWorldSignal>,
) {
    let camera_translation = camera.single().0.translation;

    // Submit load requests for chunks 8 chunks away from the camera, if they are not already loaded
    let camera_grid_x = (camera_translation.x / 16.0).floor() as i32;
    let camera_grid_y = (camera_translation.y / 16.0).floor() as i32;
    let camera_grid_z = (camera_translation.z / 16.0).floor() as i32;

    let mut world = world.single_mut();
    let mut map = &world.map;

    let mut requests: Arc<RwLock<Vec<GenerateWorldSignal>>> = Arc::new(RwLock::new(Vec::new()));

    // Concurrently check if a chunk is loaded up to 8 chunks in each direction,
    // and if not, submit a load request
    let par_iter = (-5..5).into_par_iter();
    par_iter.for_each(|x| {
        let requests = requests.clone();
        let x = x + camera_grid_x;
        for y in -5..5 {
            let y = y + camera_grid_y;
            for z in -5..5 {
                let z = z + camera_grid_z;
                if !map.chunk_loaded(x, y, z) {
                    let mut requests = requests.write().unwrap();
                    requests.push(GenerateWorldSignal { x, y, z });
                }
            }
        }
    });

    // Submit load requests
    let requests = requests.read().unwrap();
    for request in requests.iter() {
        ev_generate_world.send(request.clone());
    }

    // Store last user position
    world.prev_user_position = (
        camera_translation.x,
        camera_translation.y,
        camera_translation.z,
    );
}

fn task_generate_chunk(
    x: i32,
    y: i32,
    z: i32,
    world: &GameWorld,
    generator: &SimplePerlinGenerator,
) {
    let chunk = generator.generate_chunk(y * 16, z * 16, x * 16);
    world.map.add_chunk(chunk, x, y, z);
}

pub fn sys_generate_chunk(
    world: Query<&GameWorld>,
    mut ev_generate_world: EventReader<GenerateWorldSignal>,
) {
    let world = world.single();
    use rayon::prelude::*;
    let par_iter = ev_generate_world
        .read()
        .into_iter()
        .par_bridge()
        .into_par_iter();
    par_iter.for_each(|signal| {
        task_generate_chunk(signal.x, signal.y, signal.z, world, &world.generator);
    });
}
