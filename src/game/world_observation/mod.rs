/// # World Observation Module
/// 
/// The world observation module is responsible for observing the world and determining which chunks should be loaded and unloaded.
/// See docs/technical/world_pipeline.md for more details
/// 
/// ## Fires
/// 
/// - `ObservationLoadEvent`: When a chunk should be loaded
/// - `ObservationUnloadEvent`: When a chunk should be unloaded

/* -------------------------------------------------------------------------- */
/*                                    Misc                                    */
/* -------------------------------------------------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WorldObserverStatus {
    NeedsRefresh,
    FromPosition(WorldChunkCoordinate),
}

/// A component that observes the world.
/// 
/// When this is attached to an entity, the entity will be able to observe the world chunks, which will be loaded and unloaded as needed
#[derive(Component, Debug)]
pub struct WorldObserver {
    pub status: WorldObserverStatus,
    pub view_distance: i32,
}

impl WorldObserver {
    pub fn new() -> Self {
        Self {
            status: WorldObserverStatus::NeedsRefresh,
            view_distance: 3,
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Events                                   */
/* -------------------------------------------------------------------------- */

/// An event that signals that a chunk should be loaded (or generated), since the observation module deems it so
#[derive(Event, Debug)]
pub struct ObservationLoadEvent {
    chunk_pos: WorldChunkCoordinate
}

/// An event that signals that a chunk should be unloaded, since the observation module deems it so
#[derive(Event, Debug)]
pub struct ObservationUnloadEvent {
    chunk_pos: WorldChunkCoordinate
}

/* -------------------------------------------------------------------------- */
/*                                   Plugin                                   */
/* -------------------------------------------------------------------------- */

use bevy::{app::{App, Plugin, Startup, Update}, prelude::{Commands, Component, Entity, Event, Query, RemovedComponents, Resource, Transform}};
use egui::mutex::Mutex;

use crate::data::world::WorldChunkCoordinate;

use super::world_generator::GameWorld;

#[derive(Resource)]
pub struct WorldObservationPluginState {
    pub debug_menu: bool
}


pub struct WorldObservationPlugin {
}

impl Plugin for WorldObservationPlugin {
    fn build(&self, app: &mut App) {
        // Add events
        app.add_event::<ObservationLoadEvent>();
        app.add_event::<ObservationUnloadEvent>();
        app.insert_resource(WorldObservationPluginState {
            debug_menu: false 
        });
        app.add_systems(Startup, sys_setup);
        app.add_systems(Update, sys_update);
        
    }
}

impl Default for WorldObservationPlugin {
    fn default() -> Self {
        WorldObservationPlugin {}
    }
}

/* -------------------------------------------------------------------------- */
/*                                Basic systems                               */
/* -------------------------------------------------------------------------- */

fn sys_setup() {

}


fn sys_update(
    mut commands: Commands,
    mut observers: Query<(Entity, &mut WorldObserver, &Transform)>,
    mut removed: RemovedComponents<WorldObserver>,
    world: Query<&GameWorld>,
) {
    let world = world.single();
    let commands = Mutex::new(&mut commands);
    for (entity, mut observer, transform) in observers.iter_mut() {
        let mut distance: i32 = 0;
        {
            distance = observer.view_distance;
        }
        let status = &mut observer.status;

        match status {
            WorldObserverStatus::NeedsRefresh => {
                // Refresh ALL chunks in a DISTANCE radius
                for x in -distance..distance {
                    for y in -distance..distance {
                        for z in -distance..distance {
                            let x = x + world.prev_user_position.0 as i32;
                            let y = y + world.prev_user_position.1 as i32;
                            let z = z + world.prev_user_position.2 as i32;
                            
                            let mut commands = commands.lock();
                            commands.send_event(ObservationLoadEvent { chunk_pos: WorldChunkCoordinate { x, y, z } });
                        }
                    }
                }

                observer.status =
                    WorldObserverStatus::FromPosition(WorldChunkCoordinate::from_world(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ));
            }
            WorldObserverStatus::FromPosition(pos) => {
                // Identify current position
                let tr = transform.translation;
                let current_pos = WorldChunkCoordinate::from_world(tr.x, tr.y, tr.z);
                if *pos != current_pos {
                    // Identify chunks newly coming in and out of visibility, don't worry about chunks that stay in visibility
                    let d_x = current_pos.x - pos.x;
                    let d_y = current_pos.y - pos.y;
                    let d_z = current_pos.z - pos.z;

                    // Identify newly visible chunks
                    for x in -distance..distance {
                        for y in -distance..distance {
                            for z in -distance..distance {
                                let x = x + current_pos.x;
                                let y = y + current_pos.y;
                                let z = z + current_pos.z;
                                // AABB check
                                if x < pos.x - distance
                                    || x > pos.x + distance
                                    || y < pos.y - distance
                                    || y > pos.y + distance
                                    || z < pos.z - distance
                                    || z > pos.z + distance
                                {
                                    // Load chunk
                                    let mut commands = commands.lock();
                                    commands.send_event(ObservationLoadEvent { chunk_pos: WorldChunkCoordinate { x, y, z } });
                                }
                            }
                        }
                    }

                    // Identify no longer visible chunks
                    for x in -distance..distance {
                        for y in -distance..distance {
                            for z in -distance..distance {
                                let x = x + pos.x;
                                let y = y + pos.y;
                                let z = z + pos.z;
                                if x < current_pos.x - distance
                                    || x > current_pos.x + distance
                                    || y < current_pos.y - distance
                                    || y > current_pos.y + distance
                                    || z < current_pos.z - distance
                                    || z > current_pos.z + distance
                                {
                                    // Unload chunk
                                    let mut commands = commands.lock();
                                    commands.send_event(ObservationUnloadEvent { chunk_pos: WorldChunkCoordinate { x, y, z } });
                                }
                            }
                        }
                    }
                }

                *pos = current_pos;
            }
        }
    }
}


/* -------------------------------------------------------------------------- */
/*                              Responder systems                             */
/* -------------------------------------------------------------------------- */