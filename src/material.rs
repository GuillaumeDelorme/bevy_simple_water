use bevy_asset::{Asset, Handle};
use bevy_image::Image;
use bevy_math::Vec4;
use bevy_pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy_reflect::TypePath;
use bevy_render::render_resource::{AsBindGroup, ShaderType};
use bevy_shader::ShaderRef;

pub(crate) type WaterExtendedMaterial = ExtendedMaterial<StandardMaterial, WaterExtension>;

/// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct WaterExtension {
    /// The normal map image.
    ///
    /// Note that, like all normal maps, this must not be loaded as sRGB.
    #[texture(100)]
    #[sampler(101)]
    pub normals: Handle<Image>,

    // Parameters to the water shader.
    #[uniform(102)]
    pub settings: WaterShaderSettings,
}

impl MaterialExtension for WaterExtension {
    fn deferred_fragment_shader() -> ShaderRef {
        "embedded://bevy_simple_water/shader/water_material.wgsl".into()
    }
}

/// Shader-side parameters only.
#[derive(ShaderType, Debug, Clone)]
pub(crate) struct WaterShaderSettings {
    pub octave_vectors: [Vec4; 2],
    pub octave_scales: Vec4,
    pub octave_strengths: Vec4,
}
