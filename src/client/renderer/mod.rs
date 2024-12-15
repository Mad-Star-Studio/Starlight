use std::sync::{Mutex, RwLock};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    color::{palettes::css::WHEAT, Color},
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Camera3d, Commands, Component, Entity, EventReader, EventWriter, Mesh, Mesh3d,
        Query, Res, ResMut, Transform, With,
    },
};
use bevy_meshem::{
    prelude::{
        introduce_adjacent_chunks, mesh_grid,
        Face::{Back, Bottom, Forward, Left, Right, Top},
        MeshMD, MeshingAlgorithm,
    },
    Dimensions, VoxelRegistry,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    data::world::{World, WorldChunk, WorldChunkStatus, WorldChunkStorage},
    game::{
        registry::BlockRegistry,
        world_generator::{ChunkDroppedEvent, ChunkGeneratedEvent, GameWorld, GenerateWorldSignal},
    },
};

#[derive(Component)]
struct WorldRendererChunk {
    pub position: (i32, i32, i32),
    pub meta: MeshMD<<BlockRegistry as VoxelRegistry>::Voxel>,
    pub mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct WorldRenderer {
    pub chunks: Vec<WorldRendererChunk>,
    pub render_distance: i32,
    pub dimensions: Dimensions,
    pub material: Handle<StandardMaterial>,
}

impl WorldRenderer {
    pub fn default() -> WorldRenderer {
        WorldRenderer {
            chunks: Vec::new(),
            render_distance: 3,
            dimensions: (WorldChunk::SIZE, WorldChunk::SIZE, WorldChunk::SIZE),
            material: Handle::default(),
        }
    }
}

impl Plugin for WorldRenderer {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_setup);
        app.add_systems(Update, sys_update);
        app.add_systems(Update, sys_on_chunk_generated);
    }
}

fn adjacent_add(x: i32, y: i32, z: i32, world: &GameWorld, block_registry: &BlockRegistry) {
    let direction_table = {
        [
            (0, 0, 1),
            (0, 0, -1),
            (0, 1, 0),
            (0, -1, 0),
            (1, 0, 0),
            (-1, 0, 0),
        ]
    };
    let face_table = { [Right, Left, Top, Bottom, Forward, Back] };

    for i in 0..6 {
        let direction = direction_table[i];
        let face = face_table[i];
    }
}

fn sys_setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // let texture_handle = asset_server.load("textures/block/default_cobble.png");
    let cobble = materials.add(Color::Srgba(WHEAT));

    let mut renderer = WorldRenderer::default();
    renderer.material = cobble;
    commands.spawn(renderer);
}

fn sys_update(
    mut commands: Commands,
    block_registry: bevy::prelude::Res<BlockRegistry>,
    mut render_chunks: Query<(Entity, &mut WorldRendererChunk)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut renderer: Query<&mut WorldRenderer>,
    camera: Query<(&Camera3d, &Transform)>,
    world: Query<&GameWorld>,
    mut request_world: EventWriter<GenerateWorldSignal>,
) {
    /*let mut renderer = renderer.single_mut();
    let world = world.single();
    let block_registry = block_registry.into_inner();
    let camera_translation = camera.single().1.translation;

    let camera_grid_x = (camera_translation.x / WorldChunk::SIZE as f32).floor() as i32;
    let camera_grid_y = (camera_translation.y / WorldChunk::SIZE as f32).floor() as i32;
    let camera_grid_z = (camera_translation.z / WorldChunk::SIZE as f32).floor() as i32;

    // Identify chunks out of render distance (plus 2 for buffer), and remove them
    for mut chunk in render_chunks.iter_mut() {
        if (chunk.1.position.0 - camera_grid_x).abs() > renderer.render_distance + 16
            || (chunk.1.position.1 - camera_grid_y).abs() > renderer.render_distance + 16
            || (chunk.1.position.2 - camera_grid_z).abs() > renderer.render_distance + 16
        {
            commands.entity(chunk.0).despawn();
            // Remove chunk from renderer
            renderer.chunks.retain(|c| c.position != chunk.1.position);
        }
    }

    let render_chunks = RwLock::new(render_chunks.iter_mut().collect::<Vec<_>>());
    let meshes = RwLock::new(&mut meshes);
    let commands = RwLock::new(&mut commands);
    // Identify chunks that should be loaded in by x y z
    let par_iter = (-renderer.render_distance..renderer.render_distance).into_par_iter();
    par_iter.for_each(|x| {
        let par_iter = (-renderer.render_distance..renderer.render_distance).into_par_iter();
        par_iter.for_each(|y| {
            for z in
                camera_grid_z - renderer.render_distance..camera_grid_z + renderer.render_distance
            {
                let mut found = false;
                {
                    let render_chunks = render_chunks.read().unwrap();
                    // Check if chunk is already loaded (in ECS)
                    for chunk in render_chunks.iter() {
                        if chunk.1.position == (x, y, z) {
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    match world.map.chunk_at(x, y, z) {
                        crate::data::world::WorldChunkStatus::Stored(stored) => {
                            let r = stored.read().unwrap();
                            if r.is_loaded() {
                                let r_storage = r.unwrap().clone();
                                let r_3 = r_storage.read().unwrap();
                                let data = r_3.data();

                                match mesh_grid(
                                    renderer.dimensions,
                                    &[],
                                    data,
                                    block_registry,
                                    MeshingAlgorithm::Culling,
                                    None,
                                ) {
                                    Some((mesh, meta)) => {
                                        println!("Loaded chunk at {}, {}, {}", x, y, z);

                                        // Introduce adjacent chunks
                                        let mut meshes = meshes.write().unwrap();
                                        let mesh = meshes.add(mesh);
                                        let new_chunk = WorldRendererChunk {
                                            position: (x, y, z),
                                            mesh: mesh.clone(),
                                            meta,
                                        };
                                        let mut commands = commands.write().unwrap();
                                        commands.spawn((
                                            Mesh3d(mesh),
                                            Transform::from_translation(Vec3::new(
                                                x as f32 * WorldChunk::SIZE as f32,
                                                y as f32 * WorldChunk::SIZE as f32,
                                                z as f32 * WorldChunk::SIZE as f32,
                                            )),
                                            MeshMaterial3d(renderer.material.clone()),
                                            new_chunk,
                                        ));
                                    }
                                    None => {
                                        continue;
                                    }
                                }
                            }
                        }
                        WorldChunkStatus::Unloaded => {
                            continue;
                        }
                    }
                }
            }
        });
    });*/
}

fn sys_on_chunk_generated(
    mut commands: Commands,
    mut ev_chunk_generated: EventReader<ChunkGeneratedEvent>,
    data: ResMut<Assets<Mesh>>,
    block_registry: Res<BlockRegistry>,
    world: Query<&GameWorld>,
    renderer: Query<&WorldRenderer>,
    chunks: Query<(Entity, &mut WorldRendererChunk, &mut Mesh3d)>,
) {
    let world = world.single();
    let block_registry = block_registry.into_inner();
    let mesh_registry = Mutex::new(data);
    let commands = Mutex::new(commands);

    let par_iter = ev_chunk_generated.par_read();
    let adj_faces = [Bottom, Top, Left, Right, Forward, Back];
    let adj_offsets = [
        (0, -1, 0),
        (0, 1, 0),
        (-1, 0, 0),
        (1, 0, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];

    par_iter.for_each(|event| {
        let event = event;
        let chunk = world.map.chunk_at(event.x, event.y, event.z);
        let x = event.x;
        let y = event.y;
        let z = event.z;
        match chunk {
            WorldChunkStatus::Stored(stored) => {
                let mut loaded = false;
                {
                    let r = stored.read().unwrap();
                    loaded = r.is_loaded();
                }
                if loaded {
                    let r = stored.read().unwrap();
                    let r_arc = r.unwrap().clone();
                    let r_arc_3 = r_arc.read().unwrap();
                    let data = r_arc_3.data();
                    match mesh_grid(
                        (WorldChunk::SIZE, WorldChunk::SIZE, WorldChunk::SIZE),
                        &[],
                        data,
                        block_registry,
                        MeshingAlgorithm::Culling,
                        None,
                    ) {
                        Some((mesh, meta)) => {
                            let mut mesh = mesh;
                            let mut meta = meta;

                            // Optimize mesh by introducing adjacent chunks
                            for i in 0..6 {
                                let offset = adj_offsets[i];
                                let face = adj_faces[i];
                                let adj_chunk =
                                    world.map.chunk_at(x + offset.0, y + offset.1, z + offset.2);
                                match adj_chunk {
                                    WorldChunkStatus::Stored(adj_stored) => {
                                        let adj_r = adj_stored.read().unwrap();
                                        if adj_r.is_loaded() {
                                            let adj_r_arc = adj_r.unwrap().clone();
                                            let adj_r_arc_3 = adj_r_arc.read().unwrap();
                                            let adj_data = adj_r_arc_3.data();
                                            introduce_adjacent_chunks(
                                                block_registry,
                                                &mut mesh,
                                                &mut meta,
                                                face,
                                                adj_data,
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            // Optimize neighboring WorldRendererChunks by introducing this chunk
                            /*    for i in 0..6 {
                                let offset = adj_offsets[i];
                                let face = adj_faces[i];
                                // Find WorldRendererChunk with the same position
                                for (entity, chunk, mesh) in chunks.iter() {
                                    if chunk.position == (x + offset.0, y + offset.1, z + offset.2) {
                                //        introduce_adjacent_chunks(
                               //             block_registry,
                               //             &mut mesh_registry.lock().unwrap().get_mut(mesh).unwrap(),
                               //             &mut chunk.meta,
                               //             face,
                               //             data
                               //         );
                                    }
                                }
                            }*/

                            println!("Meshed chunk at {}, {}, {}", x, y, z);

                            let mesh = mesh_registry.lock().unwrap().add(mesh);
                            let mut commands = commands.lock().unwrap();
                            commands.spawn((
                                Mesh3d(mesh),
                                Transform::from_translation(Vec3::new(
                                    x as f32 * WorldChunk::SIZE as f32,
                                    y as f32 * WorldChunk::SIZE as f32,
                                    z as f32 * WorldChunk::SIZE as f32,
                                )),
                                MeshMaterial3d(renderer.single().material.clone()),
                                WorldRendererChunk {
                                    position: (x, y, z),
                                    mesh: Handle::default(),
                                    meta,
                                },
                            ));
                        }
                        None => {}
                    }
                }
            }
            WorldChunkStatus::Unloaded => {}
        }
    });
}

fn sys_on_chunk_dropped(
    mut commands: Commands,
    mut ev_chunk_dropped: EventReader<ChunkDroppedEvent>,
    world: Query<&GameWorld>,
    renderer: Query<&WorldRenderer>,
    chunks: Query<(Entity, &WorldRendererChunk)>,
) {
    let world = world.single();
    let renderer = renderer.single();
    // find all WorldRendererChunks that are in the dropped chunks
    for event in ev_chunk_dropped.par_read() {
        let event = event.0;
        let chunk = world.map.chunk_at(event.x, event.y, event.z);
        let x = event.x;
        let y = event.y;
        let z = event.z;

        // Search for WorldRendererChunk with the same position
        for (entity, chunk) in chunks.iter() {
            if chunk.position == (x, y, z) {
                commands.entity(entity).despawn();
            }
        }
    }
}
