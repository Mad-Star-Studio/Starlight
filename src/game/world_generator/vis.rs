use bevy::prelude::{Commands, Component, Entity, Query, Res};

use crate::data::world::{World, WorldChunkCoordinate};

const DISTANCE: u8 = 8;

/// Identifies chunks coming in and out of visibility, and unloads them when they are no longer visible.
enum WorldObserverStatus {
    NeedsRefresh,
    FromPosition(WorldChunkCoordinate),
}

#[derive(Component)]
pub struct WorldObserver {
    pub status: WorldObserverStatus,
}

pub fn sys_setup() {

}

#[inline]
fn poke_chunk(world: &dyn World) -> bool {
    false
}

pub fn sys_update(
    mut commands: Commands,
    mut query: Query<(Entity, &WorldObserver)>,
    world: Res<dyn World>,
) {
    for (entity, observer) in query.iter_mut() {
        match observer.status {
            WorldObserverStatus::NeedsRefresh => {
                
            },
            WorldObserverStatus::FromPosition(pos) => {
            },
        }
    }
}