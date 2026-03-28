//! Basic water surface example with atmosphere and SSR.

use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
    },
    light::{AtmosphereEnvironmentMapLight, SunDisk, light_consts::lux},
    pbr::DefaultOpaqueRendererMethod,
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy_pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy_render::view::Hdr;
use bevy_simple_water::{SimpleWaterPlugin, Water};

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins((DefaultPlugins, SimpleWaterPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    // Water
    commands.spawn((
        Water::ocean(),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        Transform::from_scale(Vec3::splat(100.0)),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        SunDisk::EARTH,
        Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 6.0, 0.0).looking_at(Vec3::Y * 1.8, Vec3::Y),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        AtmosphereEnvironmentMapLight::default(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        Hdr,
        Msaa::Off,
        ScreenSpaceReflections::default(),
        DeferredPrepass,
        DepthPrepass,
        MotionVectorPrepass,
        Fxaa::default(),
    ));
}
