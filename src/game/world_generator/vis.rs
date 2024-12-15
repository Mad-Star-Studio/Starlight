use crate::data::world::WorldChunkCoordinate;

/// Identifies chunks coming in and out of visibility, and unloads them when they are no longer visible.

pub struct WorldObserver {
    pub last_position: WorldChunkCoordinate,
}

pub fn sys_setup() {

}

pub fn sys_update() {

}