use crate::*;

pub fn spawn_plane(
    mut commands: Commands,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let plane_name = "su-25t";

    let plane = Plane::new(plane_name, PLANE_SPAWN_VEL);
    let plane_model_handle: Handle<Scene> =
        asset_server.load(format!("aircraft/{}/model.glb#Scene0", plane_name));
    // let plane_mesh_handle: Handle<Mesh> = asset_server.load(format!(
    //     "aircraft/{}/model.glb#Mesh0/Primitive0",
    //     plane_name
    // ));
    // let plane_texture_handle = asset_server.load(format!("aircraft/{}/texture.png", plane_name));
    // let plane_material = materials.add(StandardMaterial {
    //     base_color_texture: Some(plane_texture_handle),
    //     perceptual_roughness: 0.5,
    //     ..default()
    // });

    // Try to get the mesh asset for collider creation
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
                SceneRoot(plane_model_handle),
                Transform::from_scale(Vec3::splat(1.0))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // Rotate 180° around Y
            ));
            parent.spawn((
                Camera3d::default(),
                // Atmosphere::EARTH,
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

pub fn update_plane_readings(
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

pub fn simulate_plane(
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
