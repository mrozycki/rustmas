mod pan_orbit_camera;
mod websocket;

use bevy::prelude::*;
use bevy::window::PresentMode;
use itertools::Itertools;
use pan_orbit_camera::{pan_orbit_camera, spawn_camera};
use url::Url;

/// this component indicates what entities are LEDs
#[derive(Component, bevy::reflect::TypeUuid)]
#[uuid = "1F6B746C-C703-47AC-A70D-F531096220E8"]
struct Led(usize);

struct Points(Vec<(f32, f32, f32)>);
impl Resource for Points {}

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

fn add_lights(
    mut commands: Commands,
    points: Res<Points>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.03,
        ..Default::default()
    }));
    let leds = points
        .0
        .iter()
        .enumerate()
        .map(|(i, (x, y, z))| {
            (
                PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(*x, *y, *z),
                    ..default()
                },
                Led(i),
            )
        })
        .collect_vec();

    commands.spawn_batch(leds);
}

pub fn run(frames_endpoint: Url, points: Vec<(f32, f32, f32)>) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rustmas Visualizer".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(websocket::WebsocketPlugin::new(frames_endpoint))
        .insert_resource(Msaa::Off)
        .insert_resource(Points(points))
        .add_systems(Startup, (create_plane_and_light, spawn_camera, add_lights))
        .add_systems(Update, pan_orbit_camera)
        .run();
}
