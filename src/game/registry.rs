use bevy::prelude::{Component, Mesh, Resource};
use bevy_meshem::{VoxelMesh, VoxelRegistry};
use bevy_render::mesh::MeshVertexAttribute;

use crate::data::world::WorldNode;

#[derive(Resource)]
pub struct BlockRegistry {
    pub block: Mesh,
}

impl VoxelRegistry for BlockRegistry {
    type Voxel = WorldNode;

    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh> {
        if voxel.id == 0 {
            return VoxelMesh::Null;
        }
        VoxelMesh::NormalCube(&self.block)
    }

    fn is_covering(&self, voxel: &Self::Voxel, _side: bevy_meshem::prelude::Face) -> bool {
        return voxel.id != 0;
    }
    fn get_center(&self) -> [f32; 3] {
        return [0.5, 0.5, 0.5];
    }
    fn get_voxel_dimensions(&self) -> [f32; 3] {
        return [1.0, 1.0, 1.0];
    }
    fn all_attributes(&self) -> Vec<MeshVertexAttribute> {
        return vec![
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_UV_0,
            Mesh::ATTRIBUTE_NORMAL,
        ];
    }
}
