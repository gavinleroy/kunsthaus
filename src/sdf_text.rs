use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct SdfTextMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub sdf_texture: Handle<Image>,
    #[uniform(2)]
    pub text_color: LinearRgba,
}

impl Material for SdfTextMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_text.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
