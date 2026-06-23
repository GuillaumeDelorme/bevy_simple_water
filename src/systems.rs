use bevy_asset::{AssetServer, Assets};
use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed},
    system::{Commands, Query, Res, ResMut},
};
use bevy_pbr::MeshMaterial3d;

use crate::{
    Water,
    material::{WaterExtendedMaterial, WaterShaderSettings},
};

pub(crate) fn apply_water_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut water_materials: ResMut<Assets<WaterExtendedMaterial>>,
    added: Query<(Entity, &Water), Added<Water>>,
    changed: Query<(Entity, &Water, &MeshMaterial3d<WaterExtendedMaterial>), Changed<Water>>,
) {
    // New entities - create material
    for (entity, water) in &added {
        let handle = water_materials.add(water.to_material(&asset_server));
        commands.entity(entity).insert(MeshMaterial3d(handle));
    }

    // Existing entities - update material in place
    for (_entity, water, material_handle) in &changed {
        if let Some(mut material) = water_materials.get_mut(material_handle.0.id()) {
            material.base.base_color = water.color;
            material.base.perceptual_roughness = water.perceptual_roughness;
            material.extension.settings = WaterShaderSettings {
                octave_vectors: water.octave_vectors,
                octave_scales: water.octave_scales,
                octave_strengths: water.octave_strengths,
            };
        }
    }
}
