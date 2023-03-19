mod pan_orbit_camera;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;

use pan_orbit_camera::{pan_orbit_camera, spawn_camera};

#[derive(serde::Deserialize, bevy::reflect::TypeUuid)]
#[uuid = "7468188B-4666-499F-8F32-4DF3C244E6BD"] // <-- keep me unique
struct LightPositions {
    positions: Vec<[f32; 3]>,
}

#[derive(Resource)]
struct LightPositionsHandle(Handle<LightPositions>);

/// set up a simple 3D scene
pub fn create_plane_and_light(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn load_light_positions(mut commands: Commands, asset_server: Res<AssetServer>) {
    let light_positions = LightPositionsHandle(asset_server.load("lights.positions.json"));
    commands.insert_resource(light_positions);
}

fn add_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut light_positions: ResMut<Assets<LightPositions>>,
) {
    if !light_positions.is_empty() {
        if let Some((_, light_positions)) = light_positions.iter().next() {
            let mesh = meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.03,
                ..Default::default()
            }));
            let material = materials.add(StandardMaterial {
                base_color: Color::RED,
                ..Default::default()
            });
            for [x, y, z] in &light_positions.positions {
                commands.spawn(PbrBundle {
                    mesh: mesh.clone_weak(),
                    material: material.clone_weak(),
                    transform: Transform::from_xyz(*x, *y, *z),
                    ..default()
                });
            }
            // add one light, the only one with strong handles
            commands.spawn(PbrBundle {
                mesh,
                material,
                transform: Transform {
                    translation: Vec3::new(-1.0, -1.0, -1.0),
                    ..default()
                },
                ..default()
            });
        }
        light_positions.clear();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default()) // TODO: remove this after the project is stabilized
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(JsonAssetPlugin::<LightPositions>::new(&["positions.json"]))
        .insert_resource(Msaa::Off)
        .add_startup_system(create_plane_and_light)
        .add_startup_system(spawn_camera)
        .add_startup_system(load_light_positions)
        .add_system(pan_orbit_camera)
        .add_system(add_lights)
        .run();
}
