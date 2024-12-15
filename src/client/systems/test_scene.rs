use std::{
    f32::consts::PI,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use bevy::{
    asset::{AssetServer, Handle, LoadState, RenderAssetUsages},
    pbr::{light_consts, CascadeShadowConfigBuilder, DirectionalLight},
    prelude::{
        AlphaMode, App, Assets, Camera, Camera3d, Circle, Color, Commands, Component, Cuboid,
        DefaultPlugins, Entity, Mesh, Mesh3d, MeshMaterial3d, PointLight, Quat, Query, Res, ResMut,
        StandardMaterial, Transform, Vec3, With,
    },
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::Texture,
    },
    utils::default,
};
use bevy_meshem::{
    prelude::{
        generate_voxel_mesh, mesh_grid,
        Face::{Bottom, Top},
        MeshMD, MeshingAlgorithm,
    },
    Dimensions, VoxelMesh, VoxelRegistry,
};
use bevy_render::mesh::MeshVertexAttribute;

use crate::{
    data::world::{
        MemoryWorld, SimplePerlinGenerator, World, WorldChunk, WorldChunkStatus, WorldChunkStorage,
        WorldGenerator,
    },
    game::world_generator::GameWorld,
};

const CUBE_SIZE: usize = 16;

#[derive(Component)]
pub struct WorldComponent {
    world: MemoryWorld,
    cobble: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Meshy {
    ma: MeshingAlgorithm,
    meta: MeshMD<u32>,
}

pub fn register(mut commands: Commands) {
    // add BlockRegistry resource
    /*  let block_registry = BlockRegistry {
        block: generate_voxel_mesh(
            [1.0, 1.0, 1.0],
            [0, 0],
            [(Top, [0, 0]); 6],
            [0.5, 0.5, 0.5],
            0.05,
            Some(0.8),
            1.0,
        )
    };

    commands.insert_resource(block_registry);*/
}
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("textures/block/default_cobble.png");
    let grid = [1; CUBE_SIZE * CUBE_SIZE * CUBE_SIZE];
    let dims: Dimensions = (CUBE_SIZE, CUBE_SIZE, CUBE_SIZE);

    // check if the texture is loaded
    match asset_server.load_state(texture_handle.id()) {
        LoadState::Loaded => println!("Texture loaded"),
        _ => {}
    }

    // Set up cobble pbr texture
    let cobble = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(cobble.clone()),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Create a voxel mesh from the data
    let mut data: Vec<u8> = Vec::new();

    for _ in 0..CUBE_SIZE * CUBE_SIZE * CUBE_SIZE {
        data.push(1);
    }

    // use rand crate to generate random data
    for x in 0..CUBE_SIZE {
        for y in 0..CUBE_SIZE {
            for z in 0..CUBE_SIZE {
                data[x * CUBE_SIZE * CUBE_SIZE + y * CUBE_SIZE + z] = 0;
            }
        }
    }

    // directional 'sun' light
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 20.0, 20.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            ..default()
        }
        .build(),
    ));

    // World
    let world = MemoryWorld::new();
    commands.spawn((WorldComponent { world, cobble },));

    let mut colorful_cube = Mesh::from(Cuboid::default());
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        colorful_cube.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[r, g, b]| [(1. - *r) / 2., (1. - *g) / 2., (1. - *b) / 2., 1.])
            .collect();
        colorful_cube.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    commands.spawn((
        Mesh3d(meshes.add(colorful_cube)),
        // This is the default color, but note that vertex colors are
        // multiplied by the base color, so you'll likely want this to be
        // white if using vertex colors.
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 1.))),
        Transform::from_xyz(1.0, 0.5, 0.0),
    ));
}

pub fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world_local_query: Query<&WorldComponent>,
    mut world_query: Query<&mut GameWorld>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Camera3d, &Transform)>,
    mut mesh_query: Query<(Entity, &Transform), With<Mesh3d>>,
) {
    // Spawn in new Voxel meshes if they don't exist and are close enough to the active Camera3
    // despawn Voxel meshes that are too far away from the player

    // query the active camera3d
    let camera = query.single();
    // Get its transform component
    let camera_pos = camera.1;

    // query all the voxel meshes

    let gen = SimplePerlinGenerator::new(0);
    //let local_world = world_local_query.single();
    // let world: &MemoryWorld = &world_query.single().world;
    // Round camera position to the nearest 16
    let camera_x = (camera_pos.translation.x / 16.0).round() as i32;
    let camera_y = (camera_pos.translation.y / 16.0).round() as i32;
    let camera_z = (camera_pos.translation.z / 16.0).round() as i32;

    /*
        match mesh_grid(
            dims,
            // Automatically cull the bottom when generating the mesh
            &[Bottom],
            &grid,
            breg_res.into_inner(),
            MeshingAlgorithm::Culling,
            None,
        ) {
            Some((culled_mesh, metadata)) => {
                let culled_mesh_handle: Handle<Mesh> = meshes.add(culled_mesh.clone());
                commands.spawn((
                    Mesh3d(culled_mesh_handle),
                    MeshMaterial3d(materials.add(Color::WHITE)),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Meshy {
                        ma: MeshingAlgorithm::Culling,
                        meta: metadata,
                    },
                ));
            },
            None => {
                println!("Meshing failed");
            }
        }
    */
    /*
    for x in -4..4 {
        for y in -4..4 {
            for z in -4..4 {
                // snap camera position to the nearest 16
                let x = camera_x + (x);
                let y = camera_y + (y);
                let z = camera_z + (z);

                // Check and generate chunk data if they don't exist
                let chunk = world.chunk_at(x, y, z);
                match chunk {
                    WorldChunkStatus::Unloaded => {
                        let generated = gen.generate_chunk(x * 16, y * 16, z * 16);
                        world.add_chunk(generated, x, y, z);
                    }
                    _ => {}
                }
            }
        }
    } */
    /*
    for x in -3..3 {
        for y in -3..3 {
            for z in -3..3 {
                let x = camera_x + (x);
                let y = camera_y + (y);
                let z = camera_z + (z);

                // Check if the mesh already exists
                let exists = mesh_query.iter().any(|(_, transform)| {
                    transform.translation.x == x as f32 * 16.
                        && transform.translation.y == y as f32 * 16.
                        && transform.translation.z == z as f32 * 16.
                });

                // If the mesh doesn't exist, spawn it in
                if !exists {
                    // Ensure the chunk is in the World, with a buffer of 1 extra chunk in each direction
                    let chunk = world.chunk_at(x, y, z);
                    match chunk {
                        WorldChunkStatus::Stored(data) => {
                            let mut should_render = false;
                            if let d = data.read().unwrap() {
                                should_render = d.is_loaded();
                            }

                            if should_render {
                                let time = std::time::Instant::now();
                                if let Some(mesh) = voxel_mesh_generator(world, x, y, z) {
                                    println!("Mesh generation took {}ms", time.elapsed().as_millis());
                                    commands.spawn((
                                        Mesh3d(meshes.add(mesh)),
                                        MeshMaterial3d(local_world.cobble.clone()),
                                        Transform::from_xyz(
                                            x as f32 * 16.,
                                            y as f32 * 16.,
                                            z as f32 * 16.,
                                        ),
                                    ));
                                }
                            }
                        }
                        WorldChunkStatus::Unloaded => {}
                        _ => {}
                    }
                }
            }
        }
    } */
}
