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
    FromPosition(MapChunkCoordinate),
}

/// A component that observes the world.
/// 
/// When this is attached to an entity, the entity will be able to observe the world chunks, which will be loaded and unloaded as needed
#[derive(Debug)]
pub struct MapObserver {
    pub id: u32,
}

impl MapObserver {
    pub fn new() -> Self {
        Self { id: 0 }
    }
}

impl Component for MapObserver {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_remove(|mut world, entity, component| {
            if let Some(observer) = world.get_resource::<WorldObservationPluginState>() {
                let id;
                {
                    let this = world.get::<MapObserver>(entity).unwrap();
                    id = this.id;
                }
                if id == 0 {
                    return;
                }
                let state = &mut world.get_resource_mut::<WorldObservationPluginState>().unwrap();

                state.remove_observer(id);
            }
        });
    }
}

struct MapObserverData {
    pub status: WorldObserverStatus,
    pub view_distance: i32,
    pub position: MapCoordinate
}

/* -------------------------------------------------------------------------- */
/*                                   Events                                   */
/* -------------------------------------------------------------------------- */

/// An event that signals that a chunk should be loaded (or generated), since the observation module deems it so
#[derive(Event, Debug)]
pub struct ObservationLoadEvent {
    pub chunk_pos: MapChunkCoordinate
}

/// An event that signals that a chunk should be unloaded, since the observation module deems it so
#[derive(Event, Debug)]
pub struct ObservationUnloadEvent {
    pub chunk_pos: MapChunkCoordinate
}

/* -------------------------------------------------------------------------- */
/*                                   Plugin                                   */
/* -------------------------------------------------------------------------- */

use bevy::{app::{App, Plugin, Startup, Update}, ecs::component::StorageType, log::{debug, info, tracing_subscriber::field::debug}, prelude::{Commands, Component, Entity, Event, Query, RemovedComponents, Res, ResMut, Resource, Transform}};
use bevy_egui::{EguiContext, EguiContexts};
use egui::mutex::Mutex;

use crate::data::{MapChunkCoordinate, MapCoordinate};

use super::{perf::Profiler, world_generator::GameWorld};

/// The state of the world observation plugin, which
#[derive(Resource)]
pub struct WorldObservationPluginState {
    pub observers: Vec<(u32, MapObserverData)>,
    pub debug_menu: bool,
    pub debug_menu_z_index: f32,
}

impl WorldObservationPluginState {
    pub fn create_observer(&mut self, view_distance: i32) -> u32 {
        // Identify first number not to appear in the list
        let mut id = 0;
        for (i, _) in self.observers.iter() {
            if i.clone() != id {
                break;
            }
            id += 1;
        }

        self.observers.push((id, MapObserverData { 
            status: WorldObserverStatus::NeedsRefresh, 
            view_distance,
            position: MapCoordinate::new(0, 0, 0)
        }));
        id
    }

    pub fn remove_observer(&mut self, id: u32) {
        self.observers.retain(|(i, _)| i.clone() != id);
    }

    pub fn get_observer(&self, id: u32) -> Option<&MapObserverData> {
        for (i, observer) in &self.observers {
            if i.clone() == id {
                return Some(observer);
            }
        }
        None
    }

    pub fn get_observer_mut(&mut self, id: u32) -> Option<&mut MapObserverData> {
        for (i, observer) in &mut self.observers {
            if i.clone() == id {
                return Some(observer);
            }
        }
        None
    }

    pub fn iter(&self) -> std::slice::Iter<(u32, MapObserverData)> {
        self.observers.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<(u32, MapObserverData)> {
        self.observers.iter_mut()
    }
}

pub struct WorldObservationPlugin {
}

impl Plugin for WorldObservationPlugin {
    fn build(&self, app: &mut App) {
        // Add events
        app.add_event::<ObservationLoadEvent>();
        app.add_event::<ObservationUnloadEvent>();
        app.insert_resource(WorldObservationPluginState {
            observers: vec![],
            debug_menu: true,
            debug_menu_z_index: 0.,
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
    mut profiler: ResMut<Profiler>,
    mut state: ResMut<WorldObservationPluginState>,
    mut observers: Query<(Entity, &mut MapObserver, Option<&Transform>)>,
    removed: RemovedComponents<MapObserver>,
    mut egui_contexts: EguiContexts,
    world: Query<&GameWorld>,
) {
    let _profile = profiler.record("WorldObservation::sys_update");
    let world = world.single();
    let commands = Mutex::new(&mut commands);

    // Check for any MapObservers with id 0. If there are any, create a new observer internally
    for (entity, mut observer, transform) in observers.iter_mut() {
        if observer.id == 0 {
            observer.id = state.create_observer(4);
        }

        // Update position
        if let Some(transform) = transform {
            let position = MapCoordinate::new(transform.translation.x as i32, transform.translation.y as i32, transform.translation.z as i32);
            if let Some(observer) = state.get_observer_mut(observer.id) {
                observer.position = position;
            }
        }
    }


    for observer in state.observers.iter_mut() {
        let (_, observer) = observer;
        let distance = observer.view_distance;
        
        match observer.status {
            WorldObserverStatus::NeedsRefresh => {
                // Refresh ALL chunks in a DISTANCE radius
                for x in -distance..distance {
                    for y in -distance..distance {
                        for z in -distance..distance {
                            let x = x + world.prev_user_position.0 as i32;
                            let y = y + world.prev_user_position.1 as i32;
                            let z = z + world.prev_user_position.2 as i32;
                            
                            let mut commands = commands.lock();
                            commands.send_event(ObservationLoadEvent { chunk_pos: MapChunkCoordinate { x, y, z } });
                        }
                    }
                }
                observer.status = WorldObserverStatus::FromPosition(observer.position.get_chunk());
            }
            WorldObserverStatus::FromPosition(pos) => {
                // Identify current position
                let current_pos = observer.position.get_chunk();
                if pos != current_pos {
                    // Identify newly visible chunks
                    for x in -distance..distance {
                        for y in -distance..distance {
                            for z in -distance..distance {
                                let x = x + current_pos.x;
                                let y = y + current_pos.y;
                                let z = z + current_pos.z;

                                // Identify blocks that weren't in the pos, but are in the current_pos
                                // Use AABB to determine if the block is in the view distance
                                if x >= pos.x - distance && x <= pos.x + distance
                                    && y >= pos.y - distance && y <= pos.y + distance
                                    && z >= pos.z - distance && z <= pos.z + distance {
                                    continue;
                                }

                                let mut commands = commands.lock();
                                commands.send_event(ObservationLoadEvent { chunk_pos: MapChunkCoordinate { x, y, z } });
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

                                // Identify blocks that weren't in the current_pos, but are in the pos
                                // Use AABB to determine if the block is in the view distance
                                if x >= current_pos.x - distance && x <= current_pos.x + distance
                                    && y >= current_pos.y - distance && y <= current_pos.y + distance
                                    && z >= current_pos.z - distance && z <= current_pos.z + distance {
                                    continue;
                                }


                                let mut commands = commands.lock();
                                commands.send_event(ObservationUnloadEvent { chunk_pos: MapChunkCoordinate { x, y, z } });
                            }
                        }
                    }
                }
                observer.status = WorldObserverStatus::FromPosition(current_pos);
            }
        }
    }

    // egui debug menu
    if state.debug_menu {
        egui::Window::new("World Observation Debug Menu").show(egui_contexts.ctx_mut(), |ui| {
            ui.label(format!("Observer count: {}", state.observers.len()));

            ui.separator();

            // Z axis
            ui.horizontal(|ui| {
                ui.label("Z Axis");
                ui.drag_angle(&mut state.debug_menu_z_index);
            });

            // Use egui_plot to 
        });
    }
}


/* -------------------------------------------------------------------------- */
/*                              Responder systems                             */
/* -------------------------------------------------------------------------- */