use crate::*;
use std::f32::consts::PI;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
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
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.2,
        minimum_distance: 0.1,
        maximum_distance: 100000.0,
        ..default()
    }
    .build();
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: false,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config,
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.5, 0.0)),
    ));

    // Sky background
    commands.insert_resource(ClearColor(Color::srgb(0.6, 0.8, 1.0)));

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
    for _ in 0..cloud_amount {
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
}

pub fn camera_follow(
    gamepads: Query<&Gamepad>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    plane_query: Query<&PlaneComponent, With<Player>>, // Only query player
) {
    for mut transform in query.iter_mut() {
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
            transform.translation.z * (right_x * PI / 2.0).cos() * 1.2,
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

pub fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {:.0}", value);
            }
        }
    }
}
