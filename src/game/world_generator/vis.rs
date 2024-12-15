use bevy::prelude::{Commands, Component, Entity, Query, Res, Transform};

use crate::data::world::{World, WorldChunk, WorldChunkCoordinate, WorldChunkStatus};

use super::{ChunkDroppedEvent, GameWorld, GenerateWorldSignal};

const DISTANCE: i32 = 3;

/// Identifies chunks coming in and out of visibility, and unloads them when they are no longer visible.
enum WorldObserverStatus {
    NeedsRefresh,
    FromPosition(WorldChunkCoordinate),
}

#[derive(Component)]
pub struct WorldObserver {
    pub status: WorldObserverStatus,
}

impl WorldObserver {
    pub fn new() -> Self {
        Self {
            status: WorldObserverStatus::NeedsRefresh,
        }
    }
}

pub fn sys_setup() {}

#[inline]
fn poke_chunk(commands: &mut Commands, world: &dyn World, x: i32, y: i32, z: i32) -> bool {
    let chunk = world.chunk_at(x, y, z);
    match chunk {
        WorldChunkStatus::Stored(chunk) => false,
        WorldChunkStatus::Unloaded => {
            commands.send_event(GenerateWorldSignal { x, y, z });
            true
        }
    }
}

pub fn sys_update(
    mut commands: Commands,
    mut observers: Query<(Entity, &mut WorldObserver, &Transform)>,
    world: Query<&GameWorld>,
) {
    let world = world.single();
    for (entity, mut observer, transform) in observers.iter_mut() {
        let status = &mut observer.status;
        match status {
            WorldObserverStatus::NeedsRefresh => {
                // Refresh ALL chunks in a DISTANCE radius
                for x in -DISTANCE..DISTANCE {
                    for y in -DISTANCE..DISTANCE {
                        for z in -DISTANCE..DISTANCE {
                            let x = x + world.prev_user_position.0 as i32;
                            let y = y + world.prev_user_position.1 as i32;
                            let z = z + world.prev_user_position.2 as i32;
                            poke_chunk(&mut commands, &world.map, x, y, z);
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
                    for x in -DISTANCE..DISTANCE {
                        for y in -DISTANCE..DISTANCE {
                            for z in -DISTANCE..DISTANCE {
                                let x = x + current_pos.x;
                                let y = y + current_pos.y;
                                let z = z + current_pos.z;
                                if x < pos.x - DISTANCE
                                    || x > pos.x + DISTANCE
                                    || y < pos.y - DISTANCE
                                    || y > pos.y + DISTANCE
                                    || z < pos.z - DISTANCE
                                    || z > pos.z + DISTANCE
                                {
                                    poke_chunk(&mut commands, &world.map, x, y, z);
                                }
                            }
                        }
                    }

                    // Identify no longer visible chunks
                    for x in -DISTANCE..DISTANCE {
                        for y in -DISTANCE..DISTANCE {
                            for z in -DISTANCE..DISTANCE {
                                let x = x + pos.x;
                                let y = y + pos.y;
                                let z = z + pos.z;
                                if x < current_pos.x - DISTANCE
                                    || x > current_pos.x + DISTANCE
                                    || y < current_pos.y - DISTANCE
                                    || y > current_pos.y + DISTANCE
                                    || z < current_pos.z - DISTANCE
                                    || z > current_pos.z + DISTANCE
                                {
                                    // Unload chunk
                                    commands.send_event(ChunkDroppedEvent { x, y, z });
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
