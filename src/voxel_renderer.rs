use bevy::asset::Asset;
use bevy::pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::{AlphaMode, Color, Image, Material, Reflect};
use bevy::reflect::TypePath;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{AsBindGroup, Face, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError};

const SHADER_ASSET_PATH: &str = "shaders/voxel.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct VoxelMaterial {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(pipeline: &MaterialExtensionPipeline, descriptor: &mut RenderPipelineDescriptor, layout: &MeshVertexBufferLayout, key: MaterialExtensionKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        return Ok(());
    }
}