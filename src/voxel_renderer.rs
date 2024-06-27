use bevy::asset::Asset;
use bevy::prelude::{AlphaMode, Color, Image, Material, Reflect};
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

const SHADER_ASSET_PATH: &str = "shaders/voxel.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[uniform(0)]
    color: Color,
    alpha_mode: AlphaMode,
}

impl Default for VoxelMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}