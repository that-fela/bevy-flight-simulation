use crate::*;

pub fn spawn_enemy_plane(
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
