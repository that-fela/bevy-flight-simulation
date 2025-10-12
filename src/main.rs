mod plane {
    pub mod flight_model;
    pub mod flight_physics;
    pub mod plane;
    pub mod plane_config;
}
mod ai {
    pub mod dogfight_ai;
}
mod util;

use crate::ai::dogfight_ai::{DogfightAI, apply_ai_controls, update_dogfight_ai};
use crate::plane::plane::Plane;
use avian3d::math::PI;
use avian3d::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            PhysicsPlugins::default(),
        ))
        .add_systems(Startup, (spawn_plane, setup))
        .add_systems(Startup, spawn_enemy_plane.after(spawn_plane))
        .add_systems(
            Update,
            (
                update_dogfight_ai, // First assess situation
                apply_ai_controls,  // Then apply controls
                draw_target_vec,
                simulate,
                camera_follow,
                update_fps,
                update_plane_readings,
                // heavy_calculation,
            ),
        )
        .run();
}

#[derive(Component)]
struct PlaneComponent {
    plane: Plane,
}

#[derive(Resource)]
struct PlayerEntity(Entity);

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct PlaneReadingsText;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

pub const PLANE_SPAWN_POS: Vec3 = vec3(0.0, 3000.0, 2000.0);
pub const PLANE_SPAWN_VEL: Vec3 = vec3(0.0, 0.0, -200.0);

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let plane_name = "su-25t";

    let plane = Plane::new(plane_name, PLANE_SPAWN_VEL);
    let plane_model: Handle<Scene> =
        asset_server.load(format!("aircraft/{}/model.glb#Scene0", plane_name));
    let plane_texture_handle = asset_server.load(format!("aircraft/{}/texture.png", plane_name));
    let plane_material = materials.add(StandardMaterial {
        base_color_texture: Some(plane_texture_handle),
        perceptual_roughness: 0.5,
        ..default()
    });
    let player_entity = commands
        .spawn((
            Transform {
                translation: PLANE_SPAWN_POS,
                rotation: Quat::from_rotation_x(10_f32.to_radians()),
                ..default()
            },
            Visibility::default(),
            PlaneComponent { plane },
            Player,
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(plane_model),
                Transform::from_scale(Vec3::splat(1.0))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // Rotate 180° around Y
            ));
            parent.spawn((
                Camera3d::default(),
                Transform {
                    rotation: Quat::from_rotation_x(-30_f32.to_radians()),
                    ..default()
                },
            ));
        })
        .id();

    // Plane Readings Text
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        },
        PlaneReadingsText,
    ));

    commands.insert_resource(PlayerEntity(player_entity));
}

fn spawn_enemy_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_entity: Res<PlayerEntity>,
) {
    let plane_name = "su-25t";
    let plane = Plane::new(plane_name, vec3(0.0, 0.0, 200.0));
    let plane_model: Handle<Scene> =
        asset_server.load(format!("aircraft/{}/model.glb#Scene0", plane_name));

    commands
        .spawn((
            Transform {
                translation: Vec3::new(0.0, 3000.0, -1000.0),
                rotation: Quat::from_rotation_y(std::f32::consts::PI),
                ..default()
            },
            Visibility::default(),
            PlaneComponent { plane },
            DogfightAI::new(Some(player_entity.0)),
            Enemy,
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(plane_model),
                Transform::from_scale(Vec3::splat(1.0))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ));
        });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Dynamic physics object with a collision shape and initial angular velocity
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 4.0, -10.0),
    ));

    // Ground plane
    let grid_size = 50;
    let size = 1000.0;
    for x in -grid_size..grid_size {
        for z in -grid_size..grid_size {
            let ground_texture_handle = asset_server.load("ground.png");
            let ground_material = materials.add(StandardMaterial {
                base_color_texture: Some(ground_texture_handle),
                perceptual_roughness: 0.9,
                ..default()
            });
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(size, 0.1, size))),
                MeshMaterial3d(ground_material),
                Transform::from_xyz(x as f32 * size, 0.0, z as f32 * size),
            ));
        }
    }

    // Runway
    let grey_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        ..default()
    });
    let length = 6000.0;
    let width = 100.0;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(width, 0.2, length))),
        MeshMaterial3d(grey_material),
        Transform::from_xyz(0.0, 0.0, (-length / 2.0) + 100.0),
    ));

    // Lights
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 30000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.5, 0.0)),
    ));

    // FPS Text
    commands.spawn((
        Text::new("FPS: 0"),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgb(0.1, 0.1, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));

    // Clouds
    let cloud_altitude = 3000.0; // meters
    let cloud_amount = 10000;
    let cloud_size = 400.0; // meters
    let distibution_radius = 500000.0; // meters
    // for loop to create clouds in random positions
    for i in 0..cloud_amount {
        let x = (rand::random::<f32>() - 0.5) * distibution_radius;
        let z = (rand::random::<f32>() - 0.5) * distibution_radius;
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(cloud_size * (rand::random::<f32>() + 0.5)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_xyz(x, cloud_altitude + (rand::random::<f32>() * 1000.0), z),
        ));
    }

    // Sky background
    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)));
}

fn simulate(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut plane_query: Query<(&mut Transform, &mut PlaneComponent, Option<&Player>)>,
) {
    for (mut transform, mut plane_component, is_player) in plane_query.iter_mut() {
        let plane = &mut plane_component.plane;
        let dt = time.delta_secs();

        if is_player.is_some() {
            plane.input(&keyboard);
            for gamepad in gamepads.iter() {
                plane.gamepad_input(gamepad);
            }
        }

        for (direction, position) in &plane.flight_model.draw_vecs {
            gizmos.arrow(
                *position,
                *position + (*direction),
                Color::srgb(1.0, 0.0, 0.0),
            );
        }

        plane.simulate(dt, &mut transform);
    }
}

fn camera_follow(
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    plane_query: Query<&PlaneComponent, With<Player>>, // Only query player
) {
    for mut transform in query.iter_mut() {
        let dt = time.delta_secs();

        transform.translation = Vec3::new(0.0, 5.7, 16.9);
        transform.look_at(vec3(0.0, 2.0, 0.0), Vec3::Y);

        let Some(gamepad) = gamepads.iter().next() else {
            return;
        };

        let right_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

        let turn = vec3(
            20.0 * (right_x * PI / 2.0).sin() * 1.2,
            20.0 * (right_y * PI / 2.0).sin() * 1.2 + 5.7,
            (transform.translation.z * (right_x * PI / 2.0).cos() * 1.2),
        );

        let mut rng = rand::rng();
        let shake = plane_query
            .iter()
            .next()
            .unwrap()
            .plane
            .flight_model
            .shake_amplitude
            * 2.0;

        let offset = vec3(
            (rng.random::<f32>() - 0.5) * shake,
            (rng.random::<f32>() - 0.5) * shake,
            (rng.random::<f32>() - 0.5) * shake * 0.5,
        );

        transform.translation = transform.translation + offset + turn;
        transform.look_at(vec3(0.0, 2.0, 0.0), Vec3::Y);
    }
}

fn update_plane_readings(
    plane_query: Query<(&Transform, &PlaneComponent), With<Player>>, // Only query player
    mut text_query: Query<&mut Text, With<PlaneReadingsText>>,
) {
    if let Ok((transform, plane_component)) = plane_query.single() {
        let plane = &plane_component.plane;
        let fm = &plane.flight_model;

        for mut text in &mut text_query {
            let speed_knots = fm.velocity.length() * 1.94384;
            let altitude_feet = transform.translation.y * 3.28084;
            let throttle_in_percent = fm.left_throttle_input * 100.0;
            let throttle_out_percent = fm.left_engine_power_readout * 100.0;

            text.0 = format!(
                "
                Speed: {:.0} kt
                Alt: {:.0} ft
                Mach: {:.1}
                Throttle: {:.0}% | {:.0}%
                A: {:.2}
                g: {:.2}
                P,R,Y: {:.2}, {:.2}, {:.2}
                Gear: {:.2}
                Flaps: {:.2}
                Airbrakes: {:.2}
                Slats: {:.2}
                ",
                speed_knots,
                altitude_feet,
                fm.mach,
                throttle_in_percent,
                throttle_out_percent,
                fm.alpha,
                fm.g,
                fm.pitch_input,
                fm.roll_input,
                fm.yaw_input,
                fm.gear_pos,
                fm.flaps_pos,
                fm.airbrake_pos,
                fm.slats_pos,
            );
        }
    }
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {:.0}", value);
            }
        }
    }
}

fn draw_target_vec(
    mut gizmos: Gizmos,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for enemy_transform in enemy_query.iter() {
            let start = player_transform.translation;
            let end = enemy_transform.translation;

            // Draw arrow from player to enemy
            gizmos.arrow(
                start,
                end,
                Color::srgb(1.0, 0.0, 0.0), // Red arrow
            );
        }
    }
}
