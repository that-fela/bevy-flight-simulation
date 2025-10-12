mod plane {
    pub mod flight_model;
    pub mod flight_physics;
    pub mod plane;
    pub mod plane_config;
}
mod util;

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
        .add_systems(
            Update,
            (
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

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct PlaneReadingsText;

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let plane_name = "su-25t";

    let plane = Plane::new(plane_name);
    let plane_model: Handle<Scene> =
        asset_server.load(format!("aircraft/{}/model.glb#Scene0", plane_name));
    let plane_texture_handle = asset_server.load(format!("aircraft/{}/texture.png", plane_name));
    let plane_material = materials.add(StandardMaterial {
        base_color_texture: Some(plane_texture_handle),
        perceptual_roughness: 0.5,
        ..default()
    });
    commands
        .spawn((
            Transform {
                translation: Vec3::new(0.0, 4000.0, 0.0),
                rotation: Quat::from_rotation_x(10_f32.to_radians()),
                ..default()
            },
            Visibility::default(),
            PlaneComponent { plane },
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
        });

    // commands.spawn((
    //     Camera3d::default(),
    //     Transform {
    //         translation: Vec3::new(0.0, 10.0, -50.0),
    //         rotation: Quat::from_rotation_x(-30_f32.to_radians()),
    //         ..default()
    //     },
    // ));

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
        for y in -grid_size..grid_size {
            let ground_texture_handle = asset_server.load("ground.png");
            let ground_material = materials.add(StandardMaterial {
                base_color_texture: Some(ground_texture_handle),
                perceptual_roughness: 0.9,
                ..default()
            });
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(size, 0.1, size))),
                MeshMaterial3d(ground_material),
                Transform::from_xyz(x as f32 * size, 0.0, y as f32 * size),
            ));
        }
    }

    // Materials
    let grey_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        ..default()
    });
    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        ..default()
    });
    let length = 3000.0;
    let width = 40.0;
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

    let cloud_altitude = 3000.0; // meters
    let cloud_amount = 10000;
    let cloud_size = 200.0; // meters
    let distibution_radius = 500000.0; // meters
    // for loop to create clouds in random positions
    for i in 0..cloud_amount {
        let x = (rand::random::<f32>() - 0.5) * distibution_radius;
        let z = (rand::random::<f32>() - 0.5) * distibution_radius;
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(cloud_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_xyz(x, cloud_altitude, z),
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
    mut plane_query: Query<(&mut Transform, &mut PlaneComponent)>,
) {
    for (mut transform, mut plane_component) in plane_query.iter_mut() {
        let plane = &mut plane_component.plane;
        let dt = time.delta_secs();

        plane.input(&keyboard);
        for gamepad in gamepads.iter() {
            plane.gamepad_input(gamepad);
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
    plane_query: Query<&PlaneComponent>,
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
            transform.translation.z * (right_x * 2.0).sin() * 1.1,
            // 0.0,
            transform.translation.z * (right_y * 2.0).sin() * 1.1 + 5.7,
            transform.translation.z * (right_x * 2.0).cos() * 1.1,
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

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {:.0}", value);
            }
        }
    }
}

fn update_plane_readings(
    plane_query: Query<(&Transform, &PlaneComponent)>,
    mut text_query: Query<&mut Text, With<PlaneReadingsText>>,
) {
    if let Ok((transform, plane_component)) = plane_query.single() {
        let plane = &plane_component.plane;
        let fm = &plane.flight_model;

        for mut text in &mut text_query {
            // Convert m/s to knots (1 m/s = 1.94384 knots)
            let speed_knots = fm.velocity.length() * 1.94384;

            // Convert altitude from meters to feet (1 m = 3.28084 ft)
            let altitude_feet = transform.translation.y * 3.28084;

            // Throttle as percentage
            let throttle_in_percent = fm.left_throttle_input * 100.0;
            let throttle_out_percent = fm.left_engine_power_readout * 100.0;

            text.0 = format!(
                "
                Speed: {:.0} kt
                Alt: {:.0} ft
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

            // text.0 = format!(
            //     "s {}\nvel {:?}\nlocal vel {:?}\ncf {:?}\nalpha {}\nbeta {}\nP,R,Y: {:.2}, {:.2}, {:.2}",
            //     fm.velocity_local.length(),
            //     fm.velocity,
            //     fm.velocity_local,
            //     fm.common_force,
            //     fm.alpha,
            //     fm.beta,
            //     fm.pitch_input,
            //     fm.roll_input,
            //     fm.yaw_input,
            // );
        }
    }
}
