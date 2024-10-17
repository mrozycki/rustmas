mod api;
mod pan_orbit_camera;
mod send_mouse;

use api::RustmasApiClient;
use bevy::prelude::*;
use bevy::window::PresentMode;
use itertools::Itertools;
use pan_orbit_camera::{pan_orbit_camera, spawn_camera};
use send_mouse::send_mouse;

/// this component indicates what entities are LEDs
#[derive(Component, bevy::reflect::TypePath)]
struct Led(usize);

struct Points(Vec<(f32, f32, f32)>);
impl Resource for Points {}

fn add_lights(
    mut commands: Commands,
    points: Res<Points>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = meshes.add(Sphere::new(0.025));
    let leds = points
        .0
        .iter()
        .enumerate()
        .map(|(i, (x, y, z))| {
            (
                PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.7, 0.7, 0.7),
                        ..default()
                    }),
                    transform: Transform::from_xyz(*x, *y, -z),
                    ..default()
                },
                Led(i),
            )
        })
        .collect_vec();

    commands.spawn_batch(leds);
}

pub fn run(api_client: rustmas_webapi_client::RustmasApiClient, points: Vec<(f32, f32, f32)>) {
    let frames_endpoint = api_client.frames();
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(Points(points))
        .insert_resource(RustmasApiClient(api_client))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rustmas Visualizer".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                canvas: Some("#visualizer".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(api::WebsocketPlugin::new(frames_endpoint))
        .add_systems(Startup, (spawn_camera, add_lights))
        .add_systems(Update, pan_orbit_camera)
        .add_systems(Update, send_mouse)
        .run();
}
