use bevy_asset::AssetServer;
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_image::{
    Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
    ImageSamplerDescriptor,
};
use bevy_math::Vec4;
use bevy_pbr::{ExtendedMaterial, StandardMaterial};
use bevy_utils::default;

use crate::material::{WaterExtendedMaterial, WaterExtension, WaterShaderSettings};

/// Water surface component.
///
/// Add this alongside a [`Mesh3d`] to create a water surface.
/// The plugin automatically generates and manages the underlying material.
///
/// The water shader uses deferred rendering to animate normal-mapped waves.
/// Make sure your app uses [`DefaultOpaqueRendererMethod::deferred()`].
///
/// # Example
///
/// ```rust,ignore
/// commands.spawn((
///     Water::ocean(),
///     Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(100.0)))),
/// ));
/// ```
///
/// Settings can be modified at runtime - changes are applied automatically.
#[derive(Component, Debug, Clone)]
pub struct Water {
    /// Base color of the water surface.
    ///
    /// Darker values like `Color::srgb(0.0, 0.1, 0.3)` work well for deep ocean,
    /// while lighter greens suit shallow rivers or swamps.
    pub color: Color,

    /// Perceptual roughness of the water surface (`0.0` = mirror, `1.0` = matte).
    ///
    /// Low values (close to `0.0`) produce sharp reflections typical of calm water.
    /// Slightly higher values (e.g. `0.3`) give a softer, more diffuse look.
    pub perceptual_roughness: f32,

    /// Direction and speed of wave displacement for each octave.
    ///
    /// Each `Vec4` packs two octaves as `(u1, v1, u2, v2)`, controlling
    /// how the normal map scrolls over time. Larger values produce faster movement.
    /// Using slightly different directions per octave creates natural-looking interference.
    pub octave_vectors: [Vec4; 2],

    /// Scale (frequency) of each wave octave.
    ///
    /// Each component controls one octave. Higher values produce tighter, more frequent waves.
    /// A typical pattern doubles each octave: `Vec4::new(1.0, 2.1, 4.3, 8.4)`.
    pub octave_scales: Vec4,

    /// Strength (amplitude) of each wave octave.
    ///
    /// Each component controls one octave. Higher values produce more pronounced normals.
    /// Typically decreases with each octave: `Vec4::new(0.06, 0.03, 0.01, 0.005)`.
    pub octave_strengths: Vec4,
}

impl Default for Water {
    fn default() -> Self {
        Self::ocean()
    }
}

impl Water {
    pub(crate) fn to_material(&self, asset_server: &AssetServer) -> WaterExtendedMaterial {
        ExtendedMaterial {
            base: StandardMaterial {
                base_color: self.color,
                perceptual_roughness: self.perceptual_roughness,
                ..Default::default()
            },
            extension: WaterExtension {
                normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                    "embedded://bevy_simple_water/shader/water_normals.png",
                    |settings| {
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    },
                ),
                settings: WaterShaderSettings {
                    octave_vectors: self.octave_vectors,
                    octave_scales: self.octave_scales,
                    octave_strengths: self.octave_strengths,
                },
            },
        }
    }

    /// Deep ocean - dark blue-black, strong multi-directional waves.
    pub fn ocean() -> Self {
        Self {
            color: Color::BLACK,
            perceptual_roughness: 0.0,
            octave_vectors: [
                Vec4::new(0.080, 0.059, 0.073, -0.062),
                Vec4::new(0.153, 0.138, -0.149, -0.195),
            ],
            octave_scales: Vec4::new(1.0, 2.1, 7.9, 14.9) * 5.0,
            octave_strengths: Vec4::new(0.16, 0.18, 0.093, 0.044),
        }
    }
}
