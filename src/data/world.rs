use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex, RwLock},
};

use bevy::prelude::{Component, Resource};
use noise::{NoiseFn, Perlin};

/* -------------------------------------------------------------------------- */
/*                               World Interface                              */
/* -------------------------------------------------------------------------- */

pub type WorldNodeId = u8;

/// A node in the world.
///
/// A node is a single block in the world. It has an id that represents the type of block it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldNode {
    pub id: WorldNodeId,
}

impl WorldNode {
    pub fn new(id: WorldNodeId) -> Self {
        Self { id }
    }

    pub fn air() -> Self {
        Self { id: 0 }
    }
}

/// A chunk of the world.
///
/// A chunk is a 16x16x16 area of the world. It is the smallest unit of the world that can be loaded and unloaded.
pub struct WorldChunk {
    pub data: [WorldNode; Self::VOLUME],
}

impl WorldChunk {
    pub const SIZE: usize = 16;
    pub const VOLUME: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    pub fn new() -> Self {
        Self {
            data: [WorldNode::air(); Self::VOLUME],
        }
    }

    #[inline]
    pub fn node_at(&self, x: usize, y: usize, z: usize) -> &WorldNode {
        &self.data[x * Self::SIZE * Self::SIZE + y * Self::SIZE + z]
    }
    #[inline]
    pub fn node_at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut WorldNode {
        &mut self.data[x * Self::SIZE * Self::SIZE + y * Self::SIZE + z]
    }
    pub fn data(&self) -> &[WorldNode; Self::VOLUME] {
        &self.data
    }
}

pub enum WorldChunkStatus {
    /// A loaded chunk that can't be modified
    Stored(Arc<RwLock<WorldChunkStorage>>),
    /// A chunk that has been generated but not loaded
    Unloaded,
}

pub enum WorldChunkStorage {
    Loaded(Arc<RwLock<WorldChunk>>),
    Empty,
}

impl WorldChunkStorage {
    #[inline]
    pub fn unwrap(&self) -> Arc<RwLock<WorldChunk>> {
        match self {
            WorldChunkStorage::Loaded(chunk) => chunk.clone(),
            WorldChunkStorage::Empty => panic!("Attempted to unwrap an empty chunk"),
        }
    }

    #[inline]
    pub fn is_loaded(&self) -> bool {
        match self {
            WorldChunkStorage::Loaded(_) => true,
            WorldChunkStorage::Empty => false,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            WorldChunkStorage::Loaded(_) => false,
            WorldChunkStorage::Empty => true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldChunkCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldChunkCoordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / 16.0).floor() as i32,
            y: (y / 16.0).floor() as i32,
            z: (z / 16.0).floor() as i32,
        }
    }
}

impl Display for WorldChunkCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

///
pub trait World: Resource {
    fn add_chunk(&self, data: WorldChunkStorage, x: i32, y: i32, z: i32);
    fn unload_chunk(&self, x: i32, y: i32, z: i32);
    fn chunk_at(&self, x: i32, y: i32, z: i32) -> WorldChunkStatus;
    #[inline]
    fn chunk_loaded(&self, x: i32, y: i32, z: i32) -> bool {
        match self.chunk_at(x, y, z) {
            WorldChunkStatus::Stored(_) => true,
            WorldChunkStatus::Unloaded => false,
        }
    }
    fn save(&self, path: &str);
}

pub trait WorldGenerator {
    fn generate_chunk(&self, x: i32, y: i32, z: i32) -> WorldChunkStorage;
}

/* -------------------------------------------------------------------------- */
/*                           Simple Perlin Generator                          */
/* -------------------------------------------------------------------------- */

pub struct SimplePerlinGenerator {
    perlin: Perlin,
}

impl SimplePerlinGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
        }
    }
}

impl WorldGenerator for SimplePerlinGenerator {
    fn generate_chunk(&self, w_x: i32, w_y: i32, w_z: i32) -> WorldChunkStorage {
        //  let begin = std::time::Instant::now();
        let mut chunk = WorldChunk::new();
        let mut empty = true;

        // Precompute scaling factor once to avoid repeating it
        let scale_factor = 1.0 / 64.0;

        // Precompute the world coordinates
        let w_x = w_x as f64;
        let w_y = w_y as f64;
        let w_z = w_z as f64;

        // Get the chunk size and avoid redundant lookups
        let chunk_size = WorldChunk::SIZE as f64;

        // Iterate over each chunk's x, y, and z
        let mut n_x = 0.;
        while n_x < chunk_size {
            let mut n_y = 0.;
            while n_y < chunk_size {
                let mut n_z = 0.;
                while n_z < chunk_size {
                    // Efficient Perlin noise calculation
                    let height = self.perlin.get([
                        0.,
                        (w_y + n_y) * scale_factor,
                        (w_z + n_z) * scale_factor,
                    ]);

                    // If the height is above the threshold, set the chunk node
                    if height * 30.0 > n_x + w_x {
                        *chunk.node_at_mut(n_x as usize, n_y as usize, n_z as usize) =
                            WorldNode::new(1);
                        empty = false;
                    }
                    n_z += 1.;
                }
                n_y += 1.;
            }
            n_x += 1.;
        }

        // Print the time taken to generate the chunk
        //println!("Chunk generated in {}ms", begin.elapsed().as_millis());

        // Return chunk storage based on whether it's empty or not
        if empty {
            WorldChunkStorage::Empty
        } else {
            WorldChunkStorage::Loaded(Arc::new(RwLock::new(chunk)))
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                       In-memory World Implementation                       */
/* -------------------------------------------------------------------------- */

pub struct MemoryWorldData {
    pub chunks: Vec<(i32, i32, i32, Arc<RwLock<WorldChunkStorage>>)>,
}

impl MemoryWorldData {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn add_chunk(&mut self, data: WorldChunkStorage, x: i32, y: i32, z: i32) {
        let chunk = match data {
            WorldChunkStorage::Loaded(chunk) => {
                Arc::new(RwLock::new(WorldChunkStorage::Loaded(chunk)))
            }
            WorldChunkStorage::Empty => Arc::new(RwLock::new(WorldChunkStorage::Empty)),
        };
        self.chunks.push((x, y, z, chunk));
    }
}

#[derive(Resource)]
pub struct MemoryWorld {
    data: Arc<RwLock<MemoryWorldData>>,
    // Must last at least the lifetime of the world, but not static
    pub generate_chunk_hook:
        Box<dyn Fn(Arc<RwLock<MemoryWorldData>>, i32, i32, i32) -> bool + Send + Sync>,
}

impl MemoryWorld {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(MemoryWorldData { chunks: Vec::new() })),
            generate_chunk_hook: Box::new(|_, _, _, _| false),
        }
    }
}

impl World for MemoryWorld {
    fn unload_chunk(&self, x: i32, y: i32, z: i32) {
        let mut w = self.data.write().unwrap();
        w.chunks
            .retain(|(cx, cy, cz, _)| *cx != x || *cy != y || *cz != z);
    }

    fn add_chunk(&self, data: WorldChunkStorage, x: i32, y: i32, z: i32) {
        let mut w = self.data.write().unwrap();
        let chunk = match data {
            WorldChunkStorage::Loaded(chunk) => {
                Arc::new(RwLock::new(WorldChunkStorage::Loaded(chunk)))
            }
            WorldChunkStorage::Empty => Arc::new(RwLock::new(WorldChunkStorage::Empty)),
        };
        w.chunks.push((x, y, z, chunk));
    }

    /// Get the status of a chunk at the given coordinates.
    ///
    /// If the chunk is loaded, it will return a reference to the chunk.
    /// If the chunk is loaded and mutable, it will return a mutable reference to the chunk.
    /// If the chunk is empty, it will return an empty chunk.
    /// If the chunk is unloaded, it will return an unloaded chunk.
    fn chunk_at(&self, x: i32, y: i32, z: i32) -> WorldChunkStatus {
        {
            let r = self.data.read().unwrap();
            for (cx, cy, cz, chunk) in &r.chunks {
                if *cx == x && *cy == y && *cz == z {
                    return WorldChunkStatus::Stored(chunk.clone());
                }
            }
        }

        WorldChunkStatus::Unloaded
    }

    fn save(&self, path: &str) {
        todo!()
    }
}
