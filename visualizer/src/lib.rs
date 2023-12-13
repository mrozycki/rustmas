mod async_task_pool;
mod drawing;
mod pan_orbit_camera;
mod websocket;

use async_task_pool::TaskPool;
use bevy::prelude::*;
use bevy::window::PresentMode;
use drawing::{draw_events_generator, LastDrawPoint, RustmasApiClient};
use itertools::Itertools;
use pan_orbit_camera::{pan_orbit_camera, spawn_camera};
use url::Url;

/// this component indicates what entities are LEDs
#[derive(Component, bevy::reflect::TypeUuid)]
#[uuid = "1F6B746C-C703-47AC-A70D-F531096220E8"]
struct Led(usize);

struct Points(Vec<(f32, f32, f32)>);
impl Resource for Points {}

fn add_lights(
    mut commands: Commands,
    points: Res<Points>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.025,
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
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.7, 0.7, 0.7),
                        ..default()
                    }),
                    transform: Transform::from_xyz(*x, *y, *z),
                    ..default()
                },
                Led(i),
            )
        })
        .collect_vec();

    commands.spawn_batch(leds);
}

pub fn run(api_endpoint: Url, points: Vec<(f32, f32, f32)>) {
    let api = rustmas_webapi_client::RustmasApiClient::new(api_endpoint);
    let frames_endpoint = api.frames();

    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Points(points))
        .insert_resource(LastDrawPoint(None))
        .insert_resource(RustmasApiClient(api))
        .insert_resource(TaskPool::new())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rustmas Visualizer".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                canvas: Some("#visualizer".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(websocket::WebsocketPlugin::new(frames_endpoint))
        .add_systems(Startup, (spawn_camera, add_lights))
        .add_systems(Update, (pan_orbit_camera, draw_events_generator))
        .run();
}
