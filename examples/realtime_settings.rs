//! Demonstrates mutating `Water` every frame so the generated material updates in place.

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
use std::f32::consts::TAU;

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins((DefaultPlugins, SimpleWaterPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_water_settings)
        .run();
}

#[derive(Component)]
struct AnimatedWater;

#[derive(Default)]
struct AnimationPhase {
    wave: f32,
    hue: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    commands.spawn((
        Water::ocean(),
        AnimatedWater,
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        Transform::from_scale(Vec3::splat(100.0)),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        SunDisk::EARTH,
        Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

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

fn animate_water_settings(
    time: Res<Time>,
    mut phase: Local<AnimationPhase>,
    mut water_q: Query<&mut Water, With<AnimatedWater>>,
) {
    phase.wave = (phase.wave + time.delta_secs() * 0.22).rem_euclid(TAU);
    phase.hue = (phase.hue + time.delta_secs() * 20.0).rem_euclid(360.0);

    let swell = 0.5 + 0.5 * phase.wave.sin();
    let chop = 0.5 + 0.5 * (phase.wave * 2.0 + 0.8).sin();
    let detail = 0.5 + 0.5 * (phase.wave * 3.0 + 1.6).sin();

    for mut water in &mut water_q {
        water.color = Color::hsl(phase.hue, 0.85, 0.18 + 0.07 * swell);
        water.perceptual_roughness = 0.03 + 0.04 * (1.0 - swell * 0.6);
        water.octave_vectors = [
            Vec4::new(
                0.016 + 0.012 * swell,
                0.012 + 0.009 * chop,
                -0.014 - 0.010 * detail,
                0.010 + 0.007 * swell,
            ),
            Vec4::new(
                0.010 + 0.007 * chop,
                -0.008 - 0.006 * swell,
                0.008 + 0.005 * detail,
                -0.005 - 0.004 * chop,
            ),
        ];
        water.octave_scales = Vec4::new(
            3.2 + 0.6 * swell,
            6.8 + 0.9 * chop,
            13.5 + 1.6 * detail,
            24.5 + 2.8 * swell,
        );
        water.octave_strengths = Vec4::new(
            0.06 + 0.03 * swell,
            0.03 + 0.018 * chop,
            0.014 + 0.010 * detail,
            0.005 + 0.004 * swell,
        );
    }
}
