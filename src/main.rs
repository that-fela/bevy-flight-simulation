mod plane {
    pub mod flight_model;
    pub mod flight_physics;
    pub mod plane;
    pub mod plane_config;
}
mod ai {
    pub mod dogfight_ai;
}
mod systems {
    pub mod aircraft;
    pub mod enemy;
    pub mod game;
}
mod util;

use crate::ai::dogfight_ai::{DogfightAI, apply_ai_controls, update_dogfight_ai};
use crate::plane::plane::Plane;
use avian3d::math::PI;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    light::CascadeShadowConfigBuilder,
    prelude::*,
};
use rand::Rng;
use systems::{aircraft, enemy, game};

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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            // PhysicsPlugins::default(),
        ))
        // .add_plugins(AtmospherePlugin)
        .add_systems(Startup, (aircraft::spawn_plane, game::setup))
        .add_systems(
            Startup,
            enemy::spawn_enemy_plane.after(aircraft::spawn_plane),
        )
        .add_systems(
            Update,
            (
                update_dogfight_ai, // First assess situation
                apply_ai_controls,  // Then apply controls
                draw_target_vec,
                aircraft::simulate_plane,
                aircraft::update_plane_readings,
                game::camera_follow,
                game::update_fps,
            ),
        )
        .run();
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

            gizmos.arrow(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}
