use bevy_asset::AssetServer;
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
    ImageSamplerDescriptor,
};
use bevy_math::Vec4;
use bevy_pbr::{ExtendedMaterial, StandardMaterial};
use bevy_utils::default;

use crate::material::{WaterExtendedMaterial, WaterExtension, WaterShaderSettings};

/// Water surface component.
///
/// Add this alongside a `Mesh3d` to create a water surface.
/// The plugin automatically generates and manages the underlying material.
///
/// The water shader uses deferred rendering to animate normal-mapped waves.
/// Waves use triplanar world-space projection, so the effect does not depend on
/// the mesh UV unwrap.
/// Make sure your app uses `DefaultOpaqueRendererMethod::deferred()`.
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
    /// Each component controls one octave. Higher values produce tighter, more frequent waves
    /// in the shader's world-space projection.
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
                normals: asset_server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| {
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    })
                    .load("embedded://bevy_simple_water/shader/water_normals.png"),
                settings: WaterShaderSettings {
                    octave_vectors: self.octave_vectors,
                    octave_scales: self.octave_scales,
                    octave_strengths: self.octave_strengths,
                },
            },
        }
    }

    /// Deep ocean — dark navy, broad energetic swells.
    pub fn ocean() -> Self {
        Self {
            color: Color::srgb(0.005, 0.018, 0.055),
            perceptual_roughness: 0.08,
            octave_vectors: [
                Vec4::new(0.040, 0.026, -0.033, 0.022),
                Vec4::new(0.020, -0.016, 0.014, -0.010),
            ],
            octave_scales: Vec4::new(0.8, 1.6, 3.8, 8.0) * 4.8,
            octave_strengths: Vec4::new(0.14, 0.10, 0.05, 0.018),
        }
    }

    /// Calm pond — warm olive-green, very gentle ripples.
    pub fn pond() -> Self {
        Self {
            color: Color::srgb(0.025, 0.06, 0.035),
            perceptual_roughness: 0.18,
            octave_vectors: [
                Vec4::new(0.016, 0.010, -0.012, 0.005),
                Vec4::new(0.010, -0.012, 0.008, 0.004),
            ],
            octave_scales: Vec4::new(2.5, 5.5, 11.0, 20.0),
            octave_strengths: Vec4::new(0.035, 0.020, 0.010, 0.004),
        }
    }

    /// River — steel blue, medium directional current with cross-ripples.
    pub fn river() -> Self {
        Self {
            color: Color::srgb(0.025, 0.07, 0.12),
            perceptual_roughness: 0.08,
            octave_vectors: [
                Vec4::new(0.060, 0.010, -0.045, 0.015),
                Vec4::new(0.030, -0.008, 0.020, 0.005),
            ],
            octave_scales: Vec4::new(1.8, 3.8, 7.5, 15.0),
            octave_strengths: Vec4::new(0.13, 0.09, 0.05, 0.02),
        }
    }

    /// Swimming pool — bright cyan-blue, very calm tight ripples.
    pub fn pool() -> Self {
        Self {
            color: Color::srgb(0.04, 0.18, 0.32),
            perceptual_roughness: 0.01,
            octave_vectors: [
                Vec4::new(0.018, 0.012, -0.012, 0.006),
                Vec4::new(0.012, -0.012, 0.006, 0.006),
            ],
            octave_scales: Vec4::new(4.0, 8.0, 16.0, 28.0),
            octave_strengths: Vec4::new(0.022, 0.014, 0.008, 0.003),
        }
    }

    /// Tropical sea — saturated turquoise, lively medium waves.
    pub fn tropical() -> Self {
        Self {
            color: Color::srgb(0.002, 0.12, 0.14),
            perceptual_roughness: 0.03,
            octave_vectors: [
                Vec4::new(0.030, 0.022, -0.025, 0.018),
                Vec4::new(0.020, -0.028, 0.018, 0.012),
            ],
            octave_scales: Vec4::new(1.0, 2.2, 4.8, 9.5) * 4.5,
            octave_strengths: Vec4::new(0.11, 0.075, 0.04, 0.016),
        }
    }

    /// Calm lake — muted blue-grey, subtle wide ripples.
    pub fn lake() -> Self {
        Self {
            color: Color::srgb(0.025, 0.065, 0.12),
            perceptual_roughness: 0.10,
            octave_vectors: [
                Vec4::new(0.014, 0.009, -0.010, 0.005),
                Vec4::new(0.009, -0.011, 0.008, 0.004),
            ],
            octave_scales: Vec4::new(2.0, 4.5, 9.0, 18.0),
            octave_strengths: Vec4::new(0.050, 0.028, 0.014, 0.006),
        }
    }

    /// Natural rock pool — clear teal-green, almost still with faint detail.
    pub fn natural_pool() -> Self {
        Self {
            color: Color::srgb(0.015, 0.08, 0.065),
            perceptual_roughness: 0.06,
            octave_vectors: [
                Vec4::new(0.008, 0.005, -0.006, 0.003),
                Vec4::new(0.005, -0.008, 0.004, 0.003),
            ],
            octave_scales: Vec4::new(5.0, 10.0, 20.0, 36.0),
            octave_strengths: Vec4::new(0.016, 0.010, 0.005, 0.002),
        }
    }

    /// Swamp / marsh — murky yellow-green, soft sluggish disturbance.
    pub fn swamp() -> Self {
        Self {
            color: Color::srgb(0.04, 0.05, 0.015),
            perceptual_roughness: 0.38,
            octave_vectors: [
                Vec4::new(0.008, 0.010, -0.009, 0.005),
                Vec4::new(0.005, -0.007, 0.006, -0.003),
            ],
            octave_scales: Vec4::new(1.2, 2.4, 4.8, 9.6) * 2.2,
            octave_strengths: Vec4::new(0.065, 0.042, 0.022, 0.008),
        }
    }

    /// Arctic water — cold grey-blue, sharper choppy surface.
    pub fn arctic() -> Self {
        Self {
            color: Color::srgb(0.03, 0.05, 0.08),
            perceptual_roughness: 0.14,
            octave_vectors: [
                Vec4::new(0.040, 0.030, -0.035, 0.026),
                Vec4::new(0.055, -0.045, 0.050, 0.032),
            ],
            octave_scales: Vec4::new(0.9, 2.0, 4.5, 9.0) * 4.8,
            octave_strengths: Vec4::new(0.16, 0.11, 0.06, 0.022),
        }
    }
}
