use bevy_app::{App, Plugin, Update};
use bevy_asset::embedded_asset;
use bevy_pbr::MaterialPlugin;

use crate::{material::WaterExtendedMaterial, systems::apply_water_settings};

mod components;
mod material;
mod systems;

pub use components::Water;

/// Plugin that enables the water material and its embedded shader assets.
///
/// Add this once to your app, then spawn entities with [`Water`] and a mesh.
/// The plugin registers the extended material, embeds the WGSL shader and normal
/// map, and keeps spawned water entities synchronized with their material data.
pub struct SimpleWaterPlugin;

impl Plugin for SimpleWaterPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shader/water_material.wgsl");
        embedded_asset!(app, "shader/water_normals.png");

        app.add_plugins(MaterialPlugin::<WaterExtendedMaterial>::default());
        app.add_systems(Update, apply_water_settings);
    }
}
