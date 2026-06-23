//! Displays every built-in water preset side by side with labels.

use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    camera_controller::free_camera::{FreeCamera, FreeCameraState},
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        tonemapping::Tonemapping,
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit},
    },
    light::{AtmosphereEnvironmentMapLight, SunDisk, light_consts::lux},
    pbr::DefaultOpaqueRendererMethod,
    post_process::bloom::Bloom,
    prelude::*,
    time::{Real, Time},
    window::{CursorGrabMode, CursorOptions, Window},
};
use bevy_camera::Hdr;
use bevy_light::{Atmosphere, atmosphere::ScatteringMedium};
use bevy_pbr::{AtmosphereSettings, ScreenSpaceReflections};
use bevy_simple_water::{SimpleWaterPlugin, Water};
use std::f32::consts::PI;

type PresetEntry = (&'static str, fn() -> Water);

const PRESETS: &[PresetEntry] = &[
    ("Ocean", Water::ocean),
    ("Pool", Water::pool),
    ("Swamp", Water::swamp),
    ("Tropical", Water::tropical),
    ("Pond", Water::pond),
    ("Arctic", Water::arctic),
    ("Lake", Water::lake),
    ("Natural Pool", Water::natural_pool),
    ("River", Water::river),
];

const TILE_SIZE: f32 = 12.0;
const GAP: f32 = 1.0;
const COLUMNS: usize = 3;

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins((DefaultPlugins, SimpleWaterPlugin))
        .add_systems(Startup, (setup, spawn_controls_ui))
        .add_systems(Update, (update_labels, run_free_camera_controller))
        .run();
}

#[derive(Component)]
struct PresetLabel {
    world_position: Vec3,
}

#[derive(Component)]
struct MainCamera;

const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let water_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(TILE_SIZE / 2.0)));
    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(80.0)));

    let step = TILE_SIZE + GAP;

    for (i, (name, preset_fn)) in PRESETS.iter().enumerate() {
        let col = i % COLUMNS;
        let row = i / COLUMNS;
        let x = col as f32 * step;
        let z = row as f32 * step;

        commands.spawn((
            preset_fn(),
            Mesh3d(water_mesh.clone()),
            Transform::from_xyz(x, 0.0, z),
        ));

        commands.spawn((
            Text::new((*name).to_string()),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            PresetLabel {
                world_position: Vec3::new(x, 0.35, z),
            },
        ));
    }

    // Optional background plane to make navigation / horizon read better.
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.23, 0.20),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(step, -0.02, step),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        SunDisk::EARTH,
        Transform::IDENTITY.looking_to(Vec3::new(-0.5, -0.8, -0.5), Vec3::Y),
    ));

    let rows = PRESETS.len().div_ceil(COLUMNS);
    let center = Vec3::new(
        (COLUMNS - 1) as f32 * step / 2.0,
        0.0,
        (rows - 1) as f32 * step / 2.0,
    );

    // World atmosphere
    commands.spawn(Atmosphere::earth(
        scattering_mediums.add(ScatteringMedium::earth(256, 256)),
    ));

    // Camera tuned for this scene scale.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(center.x, 7.0, center.z + 22.0)
            .looking_at(center + Vec3::Y * 0.1, Vec3::Y),
        FreeCamera {
            sensitivity: 0.15,
            friction: 20.0,
            walk_speed: 8.0,
            run_speed: 20.0,
            scroll_factor: 0.4,
            ..default()
        },
        MainCamera,
        AtmosphereSettings::default(),
        AtmosphereEnvironmentMapLight::default(),
        Exposure { ev100: 13.0 },
        (
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            Hdr,
            Msaa::Off,
            DeferredPrepass,
            DepthPrepass,
            MotionVectorPrepass,
            ScreenSpaceReflections::default(),
            Fxaa::default(),
        ),
    ));
}

fn update_labels(
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut label_q: Query<(&PresetLabel, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };

    for (label, mut node, mut vis) in &mut label_q {
        if let Ok(vp) = camera.world_to_viewport(cam_gt, label.world_position) {
            node.left = Val::Px(vp.x - 40.0);
            node.top = Val::Px(vp.y - 10.0);
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn spawn_controls_ui(mut commands: Commands) {
    let controls = [
        ("Left Mouse", "Hold to look around"),
        ("M", "Toggle mouse capture"),
        ("W A S D", "Move"),
        ("Shift", "Run"),
        ("Scroll", "Adjust speed"),
    ];

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(16.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)))
        .with_children(|parent| {
            for (key, action) in controls {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(key.to_string()),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.8, 0.4)),
                        ));
                        row.spawn((
                            Text::new(action.to_string()),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                        ));
                    });
            }
        });
}

fn run_free_camera_controller(
    time: Res<Time<Real>>,
    mut windows: Query<(&Window, &mut CursorOptions)>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut toggle_cursor_grab: Local<bool>,
    mut mouse_cursor_grab: Local<bool>,
    mut initialized: Local<bool>,
    mut query: Query<(&mut Transform, &mut FreeCameraState, &FreeCamera), With<MainCamera>>,
) {
    let Ok((mut transform, mut state, config)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    if !*initialized {
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
        state.yaw = yaw;
        state.pitch = pitch;
        *initialized = true;
    }

    if !state.enabled {
        if *toggle_cursor_grab || *mouse_cursor_grab {
            *toggle_cursor_grab = false;
            *mouse_cursor_grab = false;

            for (_, mut cursor_options) in &mut windows {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
        }
        return;
    }

    let amount = match accumulated_mouse_scroll.unit {
        MouseScrollUnit::Line => accumulated_mouse_scroll.delta.y,
        MouseScrollUnit::Pixel => {
            accumulated_mouse_scroll.delta.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR
        }
    };
    state.speed_multiplier =
        (state.speed_multiplier + amount * config.scroll_factor).clamp(0.0, f32::MAX);

    let mut axis_input = Vec3::ZERO;
    if key_input.pressed(config.key_forward) {
        axis_input.z += 1.0;
    }
    if key_input.pressed(config.key_back) {
        axis_input.z -= 1.0;
    }
    if key_input.pressed(config.key_right) {
        axis_input.x += 1.0;
    }
    if key_input.pressed(config.key_left) {
        axis_input.x -= 1.0;
    }
    if key_input.pressed(config.key_up) {
        axis_input.y += 1.0;
    }
    if key_input.pressed(config.key_down) {
        axis_input.y -= 1.0;
    }

    let mut cursor_grab_change = false;
    if key_input.just_pressed(config.keyboard_key_toggle_cursor_grab) {
        *toggle_cursor_grab = !*toggle_cursor_grab;
        cursor_grab_change = true;
    }
    if mouse_button_input.just_pressed(config.mouse_key_cursor_grab) {
        *mouse_cursor_grab = true;
        cursor_grab_change = true;
    }
    if mouse_button_input.just_released(config.mouse_key_cursor_grab) {
        *mouse_cursor_grab = false;
        cursor_grab_change = true;
    }
    let cursor_grab = *mouse_cursor_grab || *toggle_cursor_grab;

    if axis_input != Vec3::ZERO {
        let max_speed = if key_input.pressed(config.key_run) {
            config.run_speed * state.speed_multiplier
        } else {
            config.walk_speed * state.speed_multiplier
        };
        state.velocity = axis_input.normalize() * max_speed;
    } else {
        state
            .velocity
            .smooth_nudge(&Vec3::ZERO, config.friction.clamp(0.0, f32::MAX), dt);
        if state.velocity.length_squared() < 1e-6 {
            state.velocity = Vec3::ZERO;
        }
    }

    if state.velocity != Vec3::ZERO {
        let forward = *transform.forward();
        let right = *transform.right();
        transform.translation += state.velocity.x * dt * right
            + state.velocity.y * dt * Vec3::Y
            + state.velocity.z * dt * forward;
    }

    if cursor_grab_change {
        if cursor_grab {
            for (window, mut cursor_options) in &mut windows {
                if !window.focused {
                    continue;
                }

                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
        } else {
            for (_, mut cursor_options) in &mut windows {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
        }
    }

    if accumulated_mouse_motion.delta != Vec2::ZERO && cursor_grab {
        state.pitch = (state.pitch
            - accumulated_mouse_motion.delta.y * RADIANS_PER_DOT * config.sensitivity)
            .clamp(-PI / 2.0, PI / 2.0);
        state.yaw -= accumulated_mouse_motion.delta.x * RADIANS_PER_DOT * config.sensitivity;
        transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, state.yaw, state.pitch);
    }
}
