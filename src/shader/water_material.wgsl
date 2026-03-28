// A shader that creates water ripples by overlaying 4 normal maps on top of one
// another.
//
// It only supports deferred rendering.
//
// This shader is adapted from the SSR example in the Bevy repository:
// https://github.com/bevyengine/bevy/blob/v0.18.1/assets/shaders/water_material.wgsl

#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
    pbr_fragment::pbr_input_from_standard_material,
    prepass_io::{VertexOutput, FragmentOutput},
}
#import bevy_render::globals::Globals

// Parameters to the water shader.
struct WaterSettings {
    // How much to displace each octave each frame, in the u and v directions.
    // Two octaves are packed into each `vec4`.
    octave_vectors: array<vec4<f32>, 2>,
    // How wide the waves are in each octave.
    octave_scales: vec4<f32>,
    // How high the waves are in each octave.
    octave_strengths: vec4<f32>,
}

@group(0) @binding(1) var<uniform> globals: Globals;

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var water_normals_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var water_normals_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> water_settings: WaterSettings;

// Tuned for meter-scale scenes. With the built-in presets, one base tile across
// 48 meters keeps the dominant wavelengths in a believable range on the 12m
// preset tiles while preserving broader swells for calmer water types.
const WORLD_UV_SCALE: f32 = 1.0 / 48.0;

// Samples a single octave of noise and returns the resulting normal.
fn sample_noise_octave(uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let N = textureSample(water_normals_texture, water_normals_sampler, uv).rbg * 2.0 - 1.0;
    // This isn't slerp, but it's good enough.
    return normalize(mix(vec3(0.0, 1.0, 0.0), N, strength));
}

fn sample_noise_layers(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let uv0 = uv * water_settings.octave_scales[0] + water_settings.octave_vectors[0].xy * time;
    let uv1 = uv * water_settings.octave_scales[1] + water_settings.octave_vectors[0].zw * time;
    let uv2 = uv * water_settings.octave_scales[2] + water_settings.octave_vectors[1].xy * time;
    let uv3 = uv * water_settings.octave_scales[3] + water_settings.octave_vectors[1].zw * time;
    return normalize(
        sample_noise_octave(uv0, water_settings.octave_strengths[0]) +
        sample_noise_octave(uv1, water_settings.octave_strengths[1]) +
        sample_noise_octave(uv2, water_settings.octave_strengths[2]) +
        sample_noise_octave(uv3, water_settings.octave_strengths[3])
    );
}

// Samples all four octaves of noise in world space using triplanar projection.
fn sample_noise(world_position: vec3<f32>, surface_normal: vec3<f32>, time: f32) -> vec3<f32> {
    let blend = max(abs(surface_normal), vec3(0.0001));
    let weights = blend * blend * blend * blend;
    let normalized_weights = weights / (weights.x + weights.y + weights.z);

    let axis_sign = vec3(
        select(-1.0, 1.0, surface_normal.x >= 0.0),
        select(-1.0, 1.0, surface_normal.y >= 0.0),
        select(-1.0, 1.0, surface_normal.z >= 0.0),
    );

    let uv_x = world_position.zy * WORLD_UV_SCALE;
    let uv_y = world_position.xz * WORLD_UV_SCALE;
    let uv_z = world_position.xy * WORLD_UV_SCALE;

    let noise_x = sample_noise_layers(uv_x, time);
    let noise_y = sample_noise_layers(uv_y, time);
    let noise_z = sample_noise_layers(uv_z, time);

    let world_x = normalize(vec3(
        noise_x.y * axis_sign.x,
        noise_x.z * axis_sign.x,
        noise_x.x,
    ));
    let world_y = normalize(vec3(
        noise_y.x,
        noise_y.y * axis_sign.y,
        noise_y.z * axis_sign.y,
    ));
    let world_z = normalize(vec3(
        noise_z.x,
        noise_z.z * axis_sign.z,
        noise_z.y * axis_sign.z,
    ));

    return normalize(
        world_x * normalized_weights.x +
        world_y * normalized_weights.y +
        world_z * normalized_weights.z
    );
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Create the PBR input.
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    // Bump the normal.
    pbr_input.N = sample_noise(in.world_position.xyz, normalize(pbr_input.N), globals.time);
    // Send the rest to the deferred shader.
    return deferred_output(in, pbr_input);
}
