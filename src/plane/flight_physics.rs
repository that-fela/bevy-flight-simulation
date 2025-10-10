use crate::plane::{flight_model::FlightModel, plane_config};
use crate::util::table_lerp;
use bevy::prelude::*;

const MIN_HEIGHT: f32 = 2.0;
const GRAV: Vec3 = vec3(0.0, -9.81, 0.0);

#[inline(always)]
fn local_to_global(local_vec: Vec3, world_rot: Quat) -> Vec3 {
    world_rot.normalize() * local_vec
}

#[inline(always)]
fn global_to_local(global_vec: Vec3, world_rot: Quat) -> Vec3 {
    world_rot.normalize().inverse() * global_vec
}

#[inline(always)]
fn dcs_to_bevy(vec: Vec3) -> Vec3 {
    vec3(
        vec.z,  // Your right (Z) -> Bevy's right (X)
        vec.y,  // Your up (Y) -> Bevy's up (Y)
        -vec.x, // Your forward (X) -> Bevy's forward (-Z)
    )
}

#[inline(always)]
fn bevy_to_dcs(vec: Vec3) -> Vec3 {
    vec3(
        -vec.z, // Bevy's forward (Z) -> Your forward (-X)
        vec.y,  // Bevy's up (Y) -> Your up (Y)
        vec.x,  // Bevy's right (X) -> Your right (Z)
    )
}

impl FlightModel {
    fn add_local_moment(&mut self, moment: Vec3) {
        self.common_moment += moment;
    }

    fn add_local_force(&mut self, force: Vec3, force_pos: Vec3) {
        self.common_force += force;

        let delta_pos = force_pos - self.center_of_mass;
        let delta_moment = delta_pos.cross(force);

        self.common_moment += delta_moment;
    }

    pub fn simulate(&mut self, dt: f32) {
        self.common_moment = Vec3::ZERO;
        self.common_force = Vec3::ZERO;

        // --- Aerodynamics ---
        let airspeed = self.velocity; // - wind

        let mach = airspeed.length() / self.speed_of_sound;

        let aero = &self.plane_config.aerodynamics;
        let at = &aero.tables;

        let cy_alpha = table_lerp(&at.mach, &at.Cya, mach);
        let cx0 = table_lerp(&at.mach, &at.cx0, mach);
        let cy_max = table_lerp(&at.mach, &at.CyMax, mach); // + aero.cy_flap * 0.4 * self.slats_pos;
        let alpha_max = table_lerp(&at.mach, &at.Aldop, mach);
        let omx_max = table_lerp(&at.mach, &at.OmxMax, mach);

        // let mut Cy = cy_alpha * alpha;
        // if (Cy > cy_max)
        //     Cy = cy_max;
        // if (Cy < -cy_max)
        //     Cy = -cy_max;
        // }
    }

    pub fn update_variables(&mut self) {}

    pub fn transform(&mut self, dt: f32, transform: &mut Transform) {
        let fm = self;
        let gr = transform.rotation;

        // --- Linear motion ---
        let ld_acceleration = fm.common_force / fm.current_mass;
        let ld_grav = if transform.translation.y > MIN_HEIGHT {
            bevy_to_dcs(global_to_local(GRAV, gr))
        } else {
            Vec3::ZERO
        };

        fm.velocity += (ld_acceleration + ld_grav) * dt;
        let lb_vel = dcs_to_bevy(fm.velocity);
        let gb_vel = local_to_global(lb_vel, gr);

        transform.translation += gb_vel * dt;

        if transform.translation.y <= MIN_HEIGHT {
            transform.translation.y = MIN_HEIGHT;
            fm.velocity -= ld_grav * dt;
        }

        // --- Angular motion ---
        let ld_angular_acceleration = vec3(
            fm.common_moment.x / fm.moment_of_inertia[0],
            fm.common_moment.y / fm.moment_of_inertia[1],
            fm.common_moment.z / fm.moment_of_inertia[2],
        );
        let lb_angular_acceleration = dcs_to_bevy(ld_angular_acceleration);

        fm.angular_velocity += lb_angular_acceleration * dt;

        // Convert angular velocity to quaternion rotation
        let delta_rot = Quat::from_euler(
            EulerRot::XYZ,
            fm.angular_velocity.x * dt,
            fm.angular_velocity.y * dt,
            fm.angular_velocity.z * dt,
        );

        transform.rotation = delta_rot * transform.rotation;
    }

    pub fn start_hot(&mut self) {
        // Landing gear up
        self.gear_switch = false;
        self.gear_pos = 0.0;
        self.carrier_pos = 0;

        //Engines on at 50% throttle
        self.left_engine_switch = true;
        self.left_throttle_input = 0.5;
        self.left_throttle_output = 0.5;
        self.left_engine_power_readout = 0.5;

        self.right_engine_switch = true;
        self.right_throttle_input = 0.5;
        self.right_throttle_output = 0.5;
        self.right_engine_power_readout = 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn identity_rotation_gives_same_vector() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let r = Quat::IDENTITY;
        assert_eq!(local_to_global(v, r), v);
        assert_eq!(global_to_local(v, r), v);
    }

    #[test]
    fn rotation_and_inverse_cancel_out() {
        let v = Vec3::X;
        let r = Quat::from_rotation_z(FRAC_PI_2);
        let global = local_to_global(v, r);
        let local = global_to_local(global, r);
        assert!((local - v).length() < 1e-6);
    }

    #[test]
    fn rotate_90_deg_z_axis() {
        let v = Vec3::X;
        let r = Quat::from_rotation_z(FRAC_PI_2);
        let result = local_to_global(v, r);
        assert!((result - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn rotate_back_from_global() {
        let v = Vec3::Y;
        let r = Quat::from_rotation_z(FRAC_PI_2);
        let local = global_to_local(v, r);
        assert!((local - Vec3::X).length() < 1e-6);
    }
}
