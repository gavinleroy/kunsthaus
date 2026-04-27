use std::borrow::Cow;

use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
#[bind_group_data(ShaderMaterialKey)]
pub struct ShaderMaterial {
    pub entry_point: Cow<'static, str>,
}

impl Material for ShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.entry_point = Some(key.bind_group_data.entry_point.clone());
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ShaderMaterialKey {
    entry_point: Cow<'static, str>,
}

impl From<&ShaderMaterial> for ShaderMaterialKey {
    fn from(value: &ShaderMaterial) -> Self {
        Self {
            entry_point: value.entry_point.clone(),
        }
    }
}
