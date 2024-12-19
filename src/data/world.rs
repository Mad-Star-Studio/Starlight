use std::sync::{Arc, RwLock};

use bevy::prelude::Resource;
use noise::{NoiseFn, Perlin};

/* -------------------------------------------------------------------------- */
/*                               World Interface                              */
/* -------------------------------------------------------------------------- */

pub type WorldNodeId = u8;

/// A node in the world.
///
/// A node is a single block in the world. It has an id that represents the type of block it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapBlock {
    pub id: WorldNodeId,
}

impl MapBlock {
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
pub struct MapChunk {
    pub data: [MapBlock; Self::VOLUME],
}

impl MapChunk {
    pub const SIZE: usize = 16;
    pub const VOLUME: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    pub fn new() -> Self {
        Self {
            data: [MapBlock::air(); Self::VOLUME],
        }
    }

    #[inline]
    pub fn node_at(&self, x: usize, y: usize, z: usize) -> &MapBlock {
        &self.data[x * Self::SIZE * Self::SIZE + y * Self::SIZE + z]
    }
    #[inline]
    pub fn node_at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut MapBlock {
        &mut self.data[x * Self::SIZE * Self::SIZE + y * Self::SIZE + z]
    }
    pub fn data(&self) -> &[MapBlock; Self::VOLUME] {
        &self.data
    }
}

pub enum MapChunkStatus {
    /// A loaded chunk that can't be modified
    Stored(Arc<RwLock<MapChunkStorage>>),
    /// A chunk that has been generated but not loaded
    Unloaded,
}

pub enum MapChunkStorage {
    Loaded(Arc<RwLock<MapChunk>>),
    Empty,
}

impl MapChunkStorage {
    #[inline]
    pub fn unwrap(&self) -> Arc<RwLock<MapChunk>> {
        match self {
            MapChunkStorage::Loaded(chunk) => chunk.clone(),
            MapChunkStorage::Empty => panic!("Attempted to unwrap an empty chunk"),
        }
    }

    #[inline]
    pub fn is_loaded(&self) -> bool {
        match self {
            MapChunkStorage::Loaded(_) => true,
            MapChunkStorage::Empty => false,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            MapChunkStorage::Loaded(_) => false,
            MapChunkStorage::Empty => true,
        }
    }
}

///
pub trait World: Resource {
    fn add_chunk(&self, data: MapChunkStorage, x: i32, y: i32, z: i32);
    fn unload_chunk(&self, x: i32, y: i32, z: i32);
    fn chunk_at(&self, x: i32, y: i32, z: i32) -> MapChunkStatus;
    #[inline]
    fn chunk_loaded(&self, x: i32, y: i32, z: i32) -> bool {
        match self.chunk_at(x, y, z) {
            MapChunkStatus::Stored(_) => true,
            MapChunkStatus::Unloaded => false,
        }
    }
    fn save(&self, path: &str);
}

pub trait MapGenerator {
    fn generate_chunk(&self, x: i32, y: i32, z: i32) -> MapChunkStorage;
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

impl MapGenerator for SimplePerlinGenerator {
    fn generate_chunk(&self, w_x: i32, w_y: i32, w_z: i32) -> MapChunkStorage {
        //  let begin = std::time::Instant::now();
        let mut chunk = MapChunk::new();
        let mut empty = true;

        // Precompute scaling factor once to avoid repeating it
        let scale_factor = 1.0 / 64.0;

        // Precompute the world coordinates
        let w_x = w_x as f64;
        let w_y = w_y as f64;
        let w_z = w_z as f64;

        // Get the chunk size and avoid redundant lookups
        let chunk_size = MapChunk::SIZE as f64;

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
                            MapBlock::new(1);
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
            MapChunkStorage::Empty
        } else {
            MapChunkStorage::Loaded(Arc::new(RwLock::new(chunk)))
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                       In-memory World Implementation                       */
/* -------------------------------------------------------------------------- */

pub struct MemoryWorldData {
    pub chunks: Vec<(i32, i32, i32, Arc<RwLock<MapChunkStorage>>)>,
}

impl MemoryWorldData {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn add_chunk(&mut self, data: MapChunkStorage, x: i32, y: i32, z: i32) {
        let chunk = match data {
            MapChunkStorage::Loaded(chunk) => {
                Arc::new(RwLock::new(MapChunkStorage::Loaded(chunk)))
            }
            MapChunkStorage::Empty => Arc::new(RwLock::new(MapChunkStorage::Empty)),
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

    fn add_chunk(&self, data: MapChunkStorage, x: i32, y: i32, z: i32) {
        let mut w = self.data.write().unwrap();
        let chunk = match data {
            MapChunkStorage::Loaded(chunk) => {
                Arc::new(RwLock::new(MapChunkStorage::Loaded(chunk)))
            }
            MapChunkStorage::Empty => Arc::new(RwLock::new(MapChunkStorage::Empty)),
        };
        w.chunks.push((x, y, z, chunk));
    }

    /// Get the status of a chunk at the given coordinates.
    ///
    /// If the chunk is loaded, it will return a reference to the chunk.
    /// If the chunk is loaded and mutable, it will return a mutable reference to the chunk.
    /// If the chunk is empty, it will return an empty chunk.
    /// If the chunk is unloaded, it will return an unloaded chunk.
    fn chunk_at(&self, x: i32, y: i32, z: i32) -> MapChunkStatus {
        {
            let r = self.data.read().unwrap();
            for (cx, cy, cz, chunk) in &r.chunks {
                if *cx == x && *cy == y && *cz == z {
                    return MapChunkStatus::Stored(chunk.clone());
                }
            }
        }

        MapChunkStatus::Unloaded
    }

    fn save(&self, path: &str) {
        todo!()
    }
}
