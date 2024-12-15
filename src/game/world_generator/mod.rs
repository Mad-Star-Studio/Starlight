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
        app.add_event::<ChunkUpdatedEvent>();
        app.add_event::<ChunkLoadedEvent>();
        app.add_event::<ChunkGeneratedEvent>();
        app.add_event::<ChunkDroppedEvent>();
        app.add_systems(Startup, sys_setup);
        app.add_systems(Update, sys_update);
        app.add_systems(Update, sys_generate_chunk);
        // Visibility system
        app.add_systems(Startup, vis::sys_setup);
        app.add_systems(Update, vis::sys_update);
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
#[derive(Event, Debug, Clone)]
pub struct ChunkUpdatedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Event, Debug, Clone)]
pub struct ChunkLoadedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Event, Debug, Clone)]
pub struct ChunkGeneratedEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Event, Debug, Clone)]
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
    mut ev_chunk_generated: EventWriter<ChunkGeneratedEvent>,
) {
    let world = world.single();
    use rayon::prelude::*;
    let par_iter = ev_generate_world
        .read()
        .into_iter()
        .par_bridge()
        .into_par_iter();
    let loaded_chunks: Mutex<Vec<(i32, i32, i32)>> = Mutex::new(Vec::new());
    let offsets = vec![
        (0, 0, 1),
        (0, 0, -1),
        (0, 1, 0),
        (0, -1, 0),
        (1, 0, 0),
        (-1, 0, 0),
    ];

    let ev_chunk_generated = Mutex::new(ev_chunk_generated);

    par_iter.for_each(|signal| {
        task_generate_chunk(signal.x, signal.y, signal.z, world, &world.generator);
        let mut loaded_chunks = loaded_chunks.lock().unwrap();
        loaded_chunks.push((signal.x, signal.y, signal.z));

        // Fire ChunkLoadedEvents for neighboring chunks if they have a buffer of one chunk at each cardinal direction
        for (dx, dy, dz) in offsets.iter() {
            if loaded_chunks.contains(&(signal.x + dx, signal.y + dy, signal.z + dz)) {
                ev_chunk_generated
                    .lock()
                    .unwrap()
                    .send(ChunkGeneratedEvent {
                        x: signal.x + dx,
                        y: signal.y + dy,
                        z: signal.z + dz,
                    });
            }
        }
    });

    // Fire ChunkGeneratedEvents, only if there is a buffer of one chunk at each cardinal direction
}
