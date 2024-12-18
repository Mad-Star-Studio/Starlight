use std::{fmt::{Display, Formatter}, ops::{Add, Sub}};

use bevy::prelude::Component;

use super::world::MapChunk;

/// A direction in the world.
pub enum WorldDirection {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl WorldDirection {
    pub fn as_coordinate(&self) -> MapCoordinate {
        match self {
            WorldDirection::North => MapCoordinate::north(),
            WorldDirection::South => MapCoordinate::south(),
            WorldDirection::East => MapCoordinate::east(),
            WorldDirection::West => MapCoordinate::west(),
            WorldDirection::Up => MapCoordinate::up(),
            WorldDirection::Down => MapCoordinate::down(),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                                 Coordinates                                */
/* -------------------------------------------------------------------------- */

/// A global block coordinate in the map.
/// 
/// This is a 3D coordinate that represents a block in the map.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MapCoordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_chunk(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(
            (self.x as f32 / MapChunk::SIZE as f32).floor() as i32,
            (self.y as f32 / MapChunk::SIZE as f32).floor() as i32,
            (self.z as f32 / MapChunk::SIZE as f32).floor() as i32,
        )
    }

    pub fn as_tuple(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn from_tuple(tuple: (i32, i32, i32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn one() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }

    pub fn north() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 1,
        }
    }

    pub fn south() -> Self {
        Self {
            x: 0,
            y: 0,
            z: -1,
        }
    }

    pub fn east() -> Self {
        Self {
            x: 1,
            y: 0,
            z: 0,
        }
    }
    
    pub fn west() -> Self {
        Self {
            x: -1,
            y: 0,
            z: 0,
        }
    }

    pub fn up() -> Self {
        Self {
            x: 0,
            y: 1,
            z: 0,
        }
    }

    pub fn down() -> Self {
        Self {
            x: 0,
            y: -1,
            z: 0,
        }
    }
}

impl Add for MapCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for MapCoordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Display for MapCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// A global chunk coordinate in the map.
/// 
/// This is a 3D coordinate that represents a chunk in the map.
/// As such, it refers to chunks, not blocks.
/// For instance, MapChunkCoordinate 1, 1, 1 refers to the chunk that contains blocks 16-31, 16-31, 16-31.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapChunkCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MapChunkCoordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn as_tuple(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl Display for MapChunkCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Add for MapChunkCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for MapChunkCoordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Volumes                                  */
/* -------------------------------------------------------------------------- */

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapArea {
    pub min: MapCoordinate,
    pub max: MapCoordinate,
}

impl MapArea {
    pub fn new(min: MapCoordinate, max: MapCoordinate) -> Self {
        Self { min, max }
    }

    /* --------------------------------- Checks --------------------------------- */

    pub fn contains(&self, coord: MapCoordinate) -> bool {
        coord.x >= self.min.x
            && coord.x <= self.max.x
            && coord.y >= self.min.y
            && coord.y <= self.max.y
            && coord.z >= self.min.z
            && coord.z <= self.max.z
    }

    pub fn contains_chunk(&self, coord: MapChunkCoordinate) -> bool {
        coord.x >= self.min.x / MapChunk::SIZE as i32
            && coord.x <= self.max.x / MapChunk::SIZE as i32
            && coord.y >= self.min.y / MapChunk::SIZE as i32
            && coord.y <= self.max.y / MapChunk::SIZE as i32
            && coord.z >= self.min.z / MapChunk::SIZE as i32
            && coord.z <= self.max.z / MapChunk::SIZE as i32
    }

    /* -------------------------------- Locations ------------------------------- */

    pub fn center(&self) -> MapCoordinate {
        MapCoordinate::new(
            (self.min.x + self.max.x) / 2,
            (self.min.y + self.max.y) / 2,
            (self.min.z + self.max.z) / 2,
        )
    }

    pub fn left(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.min.y, self.min.z)
    }

    pub fn right(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.max.y, self.max.z)
    }

    pub fn top(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.max.y, self.min.z)
    }

    pub fn bottom(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.min.y, self.max.z)
    }

    pub fn front(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.min.y, self.max.z)
    }

    pub fn back(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.max.y, self.min.z)
    }
    
    /* --------------------------------- Corners -------------------------------- */

    pub fn front_left_top(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.max.y, self.max.z)
    }

    pub fn front_right_top(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.max.y, self.max.z)
    }

    pub fn front_left_bottom(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.min.y, self.max.z)
    }

    pub fn front_right_bottom(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.min.y, self.max.z)
    }

    pub fn back_left_top(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.max.y, self.min.z)
    }

    pub fn back_right_top(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.max.y, self.min.z)
    }

    pub fn back_left_bottom(&self) -> MapCoordinate {
        MapCoordinate::new(self.min.x, self.min.y, self.min.z)
    }

    pub fn back_right_bottom(&self) -> MapCoordinate {
        MapCoordinate::new(self.max.x, self.min.y, self.min.z)
    }

    /* ------------------------------- Statistics ------------------------------- */

    pub fn volume(&self) -> i32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }

    pub fn chunk_volume(&self) -> i32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }
}

impl Display for MapArea {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {} - {}, {}, {})", self.min.x, self.min.y, self.min.z, self.max.x, self.max.y, self.max.z)
    }
}

/// A global chunk area in the map.
/// 
/// This is a 3D area that represents an area of chunks in the map.
/// As such, it refers to chunks, not blocks.
/// For instance, MapChunkArea 1, 1, 1 to 2, 2, 2 refers to the chunks that contain blocks 16-31, 16-31, 16-31 to 32-47, 32-47, 32-47.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapChunkArea {
    pub min: MapChunkCoordinate,
    pub max: MapChunkCoordinate,
}

impl MapChunkArea {
    pub fn new(min: MapChunkCoordinate, max: MapChunkCoordinate) -> Self {
        Self { min, max }
    }

    /* --------------------------------- Checks --------------------------------- */

    pub fn contains(&self, coord: MapChunkCoordinate) -> bool {
        coord.x >= self.min.x
            && coord.x <= self.max.x
            && coord.y >= self.min.y
            && coord.y <= self.max.y
            && coord.z >= self.min.z
            && coord.z <= self.max.z
    }

    pub fn contains_area(&self, area: MapArea) -> bool {
        area.min.x / MapChunk::SIZE as i32 >= self.min.x
            && area.max.x / MapChunk::SIZE as i32 <= self.max.x
            && area.min.y / MapChunk::SIZE as i32 >= self.min.y
            && area.max.y / MapChunk::SIZE as i32 <= self.max.y
            && area.min.z / MapChunk::SIZE as i32 >= self.min.z
            && area.max.z / MapChunk::SIZE as i32 <= self.max.z
    }

    pub fn contains_chunk_area(&self, area: MapChunkArea) -> bool {
        area.min.x >= self.min.x
            && area.max.x <= self.max.x
            && area.min.y >= self.min.y
            && area.max.y <= self.max.y
            && area.min.z >= self.min.z
            && area.max.z <= self.max.z
    }

    /* -------------------------------- Locations ------------------------------- */

    pub fn center(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(
            (self.min.x + self.max.x) / 2,
            (self.min.y + self.max.y) / 2,
            (self.min.z + self.max.z) / 2,
        )
    }

    pub fn left(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.min.y, self.min.z)
    }

    pub fn right(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.max.y, self.max.z)
    }

    pub fn top(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.max.y, self.min.z)
    }

    pub fn bottom(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.min.y, self.max.z)
    }

    pub fn front(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.min.y, self.max.z)
    }

    pub fn back(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.max.y, self.min.z)
    }

    /* --------------------------------- Corners -------------------------------- */

    pub fn front_left_top(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.max.y, self.max.z)
    }

    pub fn front_right_top(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.max.y, self.max.z)
    }

    pub fn front_left_bottom(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.min.y, self.max.z)
    }

    pub fn front_right_bottom(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.min.y, self.max.z)
    }

    pub fn back_left_top(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.max.y, self.min.z)
    }

    pub fn back_right_top(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.max.y, self.min.z)
    }

    pub fn back_left_bottom(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.min.x, self.min.y, self.min.z)
    }

    pub fn back_right_bottom(&self) -> MapChunkCoordinate {
        MapChunkCoordinate::new(self.max.x, self.min.y, self.min.z)
    }
    
    /* ------------------------------- Statistics ------------------------------- */

    pub fn volume(&self) -> i32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }

    pub fn chunk_volume(&self) -> i32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }
}