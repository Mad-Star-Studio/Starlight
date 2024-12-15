use bevy::prelude::{Component, Mesh, Resource};
use bevy_meshem::{VoxelMesh, VoxelRegistry};
use bevy_render::mesh::MeshVertexAttribute;

use crate::data::world::WorldNode;

#[derive(Resource)]
pub struct BlockRegistry {
    pub block: Mesh,
}

impl VoxelRegistry for BlockRegistry {
    /// The type of our Voxel, the example uses u16 for Simplicity but you may have a struct
    /// Block { Name: ..., etc ...}, and you'll define that as the type, but encoding the block
    /// data onto simple type like u16 or u64 is probably prefferable.
    type Voxel = WorldNode;

    /// The get_mesh function, probably the most important function in the
    /// [`VoxelRegistry`], it is what allows us to  quickly access the Mesh of each Voxel.
    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh> {
        if voxel.id == 0 {
            return VoxelMesh::Null;
        }
        VoxelMesh::NormalCube(&self.block)
    }
    /// Important function that tells our Algorithm if the Voxel is "full", for example, the Air
    /// in minecraft is not "full", but it is still on the chunk data, to singal there is nothing.
    fn is_covering(&self, voxel: &Self::Voxel, _side: bevy_meshem::prelude::Face) -> bool {
        return voxel.id != 0;
    }
    /// The center of the Mesh, out mesh is defined in src/default_block.rs, just a constant.
    fn get_center(&self) -> [f32; 3] {
        return [0.5, 0.5, 0.5];
    }
    /// The dimensions of the Mesh, out mesh is defined in src/default_block.rs, just a constant.
    fn get_voxel_dimensions(&self) -> [f32; 3] {
        return [1.0, 1.0, 1.0];
    }
    /// The attributes we want to take from out voxels, note that using a lot of different
    /// attributes will likely lead to performance problems and unpredictible behaviour.
    /// We chose these 3 because they are very common, the algorithm does preserve UV data.
    fn all_attributes(&self) -> Vec<MeshVertexAttribute> {
        return vec![
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_UV_0,
            Mesh::ATTRIBUTE_NORMAL,
        ];
    }
}
