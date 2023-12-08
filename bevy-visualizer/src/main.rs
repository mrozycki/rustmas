mod pan_orbit_camera;
mod websocket;

use bevy::prelude::*;
use bevy::window::PresentMode;

use csv::ReaderBuilder;
use pan_orbit_camera::{pan_orbit_camera, spawn_camera};

/// this component indicates what entities are LEDs
#[derive(Component, bevy::reflect::TypeUuid)]
#[uuid = "1F6B746C-C703-47AC-A70D-F531096220E8"]
struct Led(usize);

static LIGHTS_CSV: &[u8] = include_bytes!("../assets/lights.csv");

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.03,
        ..Default::default()
    }));
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(false)
        .from_reader(LIGHTS_CSV);
    let leds = rdr
        .deserialize()
        .filter_map(|record: Result<(f32, f32, f32), _>| record.ok())
        .enumerate()
        .map(|(i, (x, y, z))| {
            (
                PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_xyz(x, y, z),
                    ..default()
                },
                Led(i),
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(leds);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rustmas Visualizer".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(websocket::WebsocketPlugin::new(
            "ws://127.0.0.1:8081/frames",
        ))
        .insert_resource(Msaa::Off)
        .add_startup_system(create_plane_and_light)
        .add_startup_system(spawn_camera)
        .add_startup_system(add_lights)
        .add_system(pan_orbit_camera)
        .run();
}
