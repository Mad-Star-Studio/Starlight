/* -------------------------------------------------------------------------- */
/*                                   Plugin                                   */
/* -------------------------------------------------------------------------- */

use bevy::{app::{App, Startup, Update}, prelude::{Commands, Component, Event, EventReader, IntoSystemConfigs, ResMut, Resource}};

use crate::data::MapChunkCoordinate;

use super::perf::Profiler;

pub struct WorldManagerPlugin {

}

impl Default for WorldManagerPlugin {
    fn default() -> Self {
        WorldManagerPlugin {}
    }
}

impl bevy::prelude::Plugin for WorldManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldManagerLoadRequest>();
        app.add_systems(Startup, sys_setup);
        app.add_systems(Update, (
            // These should run sequentially.
            sys_world_manager_unload_event,
            sys_world_manager_load_event,
            sys_update
        ).chain());
    }
}

/* -------------------------------------------------------------------------- */
/*                                    Data                                    */
/* -------------------------------------------------------------------------- */

enum WorldManagerChunkFlag {
    Ready,
    Buffer
}

enum WorldManagerChunkState {
    Loaded(WorldManagerChunkFlag),
    QueueLoad,
    QueueUnload,
    Unloaded,
}

#[derive(Resource, Debug)]
pub struct WorldManager {
    
}

/* -------------------------------------------------------------------------- */
/*                                   Events                                   */
/* -------------------------------------------------------------------------- */

#[derive(Event, Debug, Clone)]
pub struct WorldManagerLoadRequest {
    chunk_pos: MapChunkCoordinate
}

#[derive(Event, Debug, Clone)]
pub struct WorldManagerUnloadRequest {
    chunk_pos: MapChunkCoordinate
}

/* -------------------------------------------------------------------------- */
/*                              Scheduled systems                             */
/* -------------------------------------------------------------------------- */

fn sys_setup(mut commands: Commands) {
    commands.insert_resource(WorldManager {});
}

fn sys_update(
    mut commands: Commands,
    mut world_manager: ResMut<WorldManager>,
    mut profiler: ResMut<Profiler>,
    ) {
    let _profiler = profiler.record("WorldManager::sys_update");
}

/* -------------------------------------------------------------------------- */
/*                              Responder systems                             */
/* -------------------------------------------------------------------------- */

fn sys_world_manager_load_event(
    mut commands: Commands, 
    world_manager: ResMut<WorldManager>,
    mut event: EventReader<WorldManagerLoadRequest>
) {
    let load_events = event.read();
    for _ in load_events {

    }
}

fn sys_world_manager_unload_event(
    mut commands: Commands,
    world_manager: ResMut<WorldManager>,
    mut event: EventReader<WorldManagerUnloadRequest>
) {
}

/* -------------------------------------------------------------------------- */
/*                               Misc functions                               */
/* -------------------------------------------------------------------------- */