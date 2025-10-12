use std::f64::consts::E;
use std::process::exit;

use crate::plane::{flight_model::FlightModel, plane_config};
use crate::util::{actuator, iter_or_exit, limit, rad, rescale, table_lerp};
use bevy::math::ops::sqrt;
use bevy::prelude::*;

const MIN_HEIGHT: f32 = 2.0;
const GRAV: Vec3 = vec3(0.0, -9.81, 0.0);
pub const ALTITUDE_M: [f32; 16] = [
    0.0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0, 3000.0, 3500.0, 4000.0, 4500.0, 5000.0, 6000.0,
    7000.0, 8000.0, 9000.0, 10000.0,
];
pub const AIR_DENSITY_KG_PER_M3: [f32; 16] = [
    1.225, 1.167, 1.112, 1.058, 1.007, 0.957, 0.909, 0.863, 0.819, 0.777, 0.736, 0.660, 0.590,
    0.525, 0.467, 0.414,
];

#[inline(always)]
fn local_to_global(local_vec: Vec3, world_rot: Quat) -> Vec3 {
    world_rot.normalize() * local_vec
}

#[inline(always)]
fn global_to_local(global_vec: Vec3, world_rot: Quat) -> Vec3 {
    world_rot.normalize().inverse() * global_vec
}

fn sec_order_sim(x: f32, dx: f32, spring: f32, damp: f32) -> f32 {
    let damp_f = -dx * damp;
    let total = damp_f + x * spring;
    total
}

impl FlightModel {
    fn add_local_moment(&mut self, moment: Vec3) {
        self.common_moment += moment;
    }

    fn add_local_force_draw(&mut self, force: Vec3, force_pos: Vec3, t: &Transform) {
        let gb_pos = local_to_global((force_pos), t.rotation) + t.translation;
        let rot = local_to_global((force), t.rotation);
        self.draw_vecs.push((rot, gb_pos));

        self.add_local_force(force, force_pos);
    }

    fn add_local_force(&mut self, force: Vec3, force_pos: Vec3) {
        self.common_force += force;

        let delta_pos = force_pos - self.center_of_mass;
        let delta_moment = delta_pos.cross(force);

        self.common_moment += delta_moment;
    }

    fn sim_engine(&mut self, mach: &f32, dt: &f32, t: &Transform) {
        let et = &self.plane_config.engine.tables;
        let max_dry_thrust = table_lerp(&et.mach, &et.max_thrust, *mach);

        self.left_throttle_input = limit(self.left_throttle_input, 0.0, 1.0);
        self.right_throttle_input = limit(self.right_throttle_input, 0.0, 1.0);

        // Left engine
        self.left_throttle_output = limit(
            table_lerp(
                &et.throttle_input,
                &et.engine_power,
                self.left_throttle_input,
            ),
            0.1,
            1.0,
        );
        self.left_engine_power_readout = limit(
            table_lerp(
                &et.throttle_input,
                &et.engine_power_readout,
                self.left_throttle_input,
            ),
            0.0,
            1.0,
        );

        // Right engine
        self.right_throttle_output = limit(
            table_lerp(
                &et.throttle_input,
                &et.engine_power,
                self.right_throttle_input,
            ),
            0.1,
            1.0,
        );
        self.right_engine_power_readout = limit(
            table_lerp(
                &et.throttle_input,
                &et.engine_power_readout,
                self.right_throttle_input,
            ),
            0.0,
            1.0,
        );

        let left_thrust_force =
            self.left_throttle_output * max_dry_thrust * self.engine_alt_effect * 0.5;
        let right_thrust_force =
            self.right_throttle_output * max_dry_thrust * self.engine_alt_effect * 0.5;

        // Engine shutdown
        if self.internal_fuel <= 0.0 || self.altitude_asl > 20_000.0 {
            self.left_thrust_force = 0.0;
            self.right_thrust_force = 0.0;
            self.left_engine_switch = false;
            self.right_engine_switch = false;

            self.left_engine_power_readout =
                actuator(self.left_engine_power_readout, 0.0, -dt / 10.0, dt / 10.0);
            self.right_engine_power_readout =
                actuator(self.right_engine_power_readout, 0.0, -dt / 10.0, dt / 10.0);
        }

        self.add_local_force(vec3(0.0, 0.0, -left_thrust_force), self.left_engine_pos);
        self.add_local_force(vec3(0.0, 0.0, -right_thrust_force), self.right_engine_pos);
    }

    pub fn update_wings(
        &mut self,
        alpha_max: f32,
        lift: f32,
        drag: f32,
        q: f32,
        s: f32,
        aos: f32,
        aoa: f32,
        t: &Transform,
        cy_tail: f32,
    ) {
        if (self.alpha.abs() / alpha_max) >= 0.75 {
            self.left_wing_pos.z = self.center_of_mass.z
                + 0.7
                + (limit(
                    (self.alpha.abs() / (alpha_max * 1.1)).powi(3) / 2000.0,
                    0.0,
                    self.plane_config.basic.length / 3.0,
                ) + limit(-aos * 10.0, 0.0, 1.0));

            self.right_wing_pos.z = self.center_of_mass.z
                + 0.7
                + (limit(
                    (self.alpha.abs() / (alpha_max * 1.1)).powi(3) / 2000.0,
                    0.0,
                    self.plane_config.basic.length / 3.0,
                ) + limit(aos * 10.0, 0.0, 1.0));
        } else {
            self.left_wing_pos.z = self.center_of_mass.z + 0.7;
            self.right_wing_pos.z = self.center_of_mass.z + 0.7;
        }

        let left_wing_forces = vec3(
            0.0,
            lift * ((-aos / 2.0).sin() / 2.0 + 1.0) * q * (s / 2.0),
            drag * ((-aos / 2.0).sin() + 1.0) * q * (s / 2.0),
        );

        let right_wing_forces = vec3(
            0.0,
            lift * ((-aos / 2.0).sin() / 2.0 + 1.0) * q * (s / 2.0),
            drag * ((-aos / 2.0).sin() + 1.0) * q * (s / 2.0),
        );

        let tail_force = vec3(
            cy_tail * aoa.cos() * q * (s / 2.0),
            0.0,
            -(-cy_tail).powi(3) * aoa.sin() * (s / 2.0) * q,
        );

        // self.add_local_force_draw(tail_force, self.tail_pos, t);
        // self.add_local_force_draw(right_wing_forces, self.right_wing_pos, t);
        // self.add_local_force_draw(left_wing_forces, self.left_wing_pos, t);
        self.add_local_force(left_wing_forces, self.left_wing_pos);
        self.add_local_force(right_wing_forces, self.right_wing_pos);
        self.add_local_force(tail_force, self.tail_pos);
    }

    pub fn update_elevator(&mut self, aoa: f32, q: f32, mach: &f32, t: &Transform) {
        if self.pitch_analog {
            self.pitch_input = limit(self.pitch_input, -1.0, 1.0);
        } else {
            if self.pitch_discrete == 1 {
                self.pitch_input = (self.pitch_input + 0.0035).min(1.0);
            }
            if self.pitch_discrete == 0 && self.pitch_input > 0.5 && self.pitch_input > 0.7 {
                self.pitch_input *= 0.98;
            }
            if self.pitch_discrete == -1 {
                self.pitch_input = (self.pitch_input - 0.0035).max(-1.0);
            }
            if self.pitch_discrete == 0 && self.pitch_input < -0.5 && self.pitch_input < -0.5 {
                self.pitch_input *= 0.98;
            }
        }

        self.pitch_trim = limit(self.pitch_trim, -0.3, 0.3);
        self.elevator_command = limit(
            actuator(
                self.elevator_command,
                self.pitch_input + self.pitch_trim,
                -0.0125,
                0.0125,
            ),
            -1.0,
            1.0,
        );

        let elevator_deflection = -(rescale(self.elevator_command + 0.15, rad(-25.0), rad(35.0)))
            * 14.0
            * (aoa / 2.0).cos();
        let pitch_stability = (aoa + (aoa / 2.0).sin() / 2.0) + (self.pitch_rate * 2.0);

        let f = vec3(
            0.0,
            ((elevator_deflection
                * limit(
                    1.0 - ((mach + self.plane_config.basic.mach_max * 0.4) / 3.0).sqrt(),
                    0.001,
                    1.0,
                ))
                + (pitch_stability * (mach / 2.0 + 1.0)))
                * q
                / 2.0,
            0.0,
        );
        // self.add_local_force_draw(f, self.elevator_pos, t);
        self.add_local_force(f, self.elevator_pos);
    }

    pub fn update_roll(&mut self, aos: f32, aoa: f32, q: f32, t: &Transform) {
        if self.roll_analog {
            self.roll_input = limit(self.roll_input, -1.0, 1.0);
        } else {
            if self.roll_discrete == 1 {
                self.roll_input = (self.roll_input + 0.004).min(1.0);
            }
            if self.roll_discrete == -1 {
                self.roll_input = (self.roll_input - 0.004).max(-1.0);
            }
            if self.roll_discrete == 0 {
                self.roll_input *= 0.9;
            }
        }

        self.roll_trim = limit(self.roll_trim, -0.3, 0.3);
        self.aileron_command = limit(
            actuator(
                self.aileron_command,
                self.roll_input + self.roll_trim,
                -0.02,
                0.02,
            ),
            -1.0,
            1.0,
        );

        let aileron_deflection = rescale(self.aileron_command, rad(-30.0), rad(30.0)) * 4.0;
        let roll_stability = -self.roll_rate
            * (((aoa + 0.5).abs() * (aos + 0.5).abs()) + 1.0)
            * (5.0 / self.wingspan)
            + ((self.roll.sin() / 2.0) * (aoa / 2.0).abs());

        // self.add_local_force_draw(
        //     vec3(0.0, (aileron_deflection + roll_stability) * q, 0.0),
        //     self.left_aileron_pos,
        //     t,
        // );
        // self.add_local_force_draw(
        //     vec3(0.0, -(aileron_deflection + roll_stability) * q, 0.0),
        //     self.right_aileron_pos,
        //     t,
        // );
        self.add_local_force(
            vec3(0.0, (aileron_deflection + roll_stability) * q, 0.0),
            self.left_aileron_pos,
        );
        self.add_local_force(
            vec3(0.0, -(aileron_deflection + roll_stability) * q, 0.0),
            self.right_aileron_pos,
        );
    }

    pub fn update_yaw(&mut self, aos: f32, q: f32, t: &Transform) {
        if self.yaw_analog {
            self.yaw_input = limit(self.yaw_input, -1.0, 1.0);
        } else {
            if self.yaw_discrete == 1 {
                self.yaw_input = (self.yaw_input + 0.0035).min(1.0);
            }
            if self.yaw_discrete == -1 {
                self.yaw_input = (self.yaw_input - 0.0035).max(-1.0);
            }
            if self.yaw_discrete == 0 {
                self.yaw_input *= 0.9;
            }
        }

        self.yaw_trim = limit(self.yaw_trim, -0.2, 0.2);
        self.rudder_command = limit(
            actuator(
                self.rudder_command,
                self.yaw_input + self.yaw_trim,
                -0.012,
                0.012,
            ),
            -1.0,
            1.0,
        );

        let rudder_deflection = rescale(self.rudder_command, rad(-30.0), rad(30.0)) * 1.5;
        let yaw_stability = -((aos * 2.0) + self.yaw_rate);

        self.add_local_force(
            vec3((rudder_deflection + yaw_stability) * q, 0.0, 0.0),
            self.rudder_pos,
        );
        self.add_local_force(
            vec3((rudder_deflection + yaw_stability) * q, 0.0, 0.0),
            self.rudder_pos,
        );
    }

    pub fn update_other(&mut self, q: f32, omx_max: f32, mach: f32, aos: f32) {
        let roll_yaw_moment = -(self.roll_rate / 2.0) * (q + 1e5 * 0.5); // Subtle yaw moment to keep stable in sharp turns
        self.add_local_moment(vec3(0.0, roll_yaw_moment, 0.0));

        let roll_rate_limiter = -self.roll_rate
            * limit(
                (limit(self.roll_rate.abs() / (omx_max + 0.1), 0.0001, 2.0)).powi(6)
                    * (q + q + 1e5 * 0.3),
                -1e7,
                1e7,
            );

        self.add_local_moment(Vec3::new(roll_rate_limiter, 0.0, 0.0));

        let yaw_rate_limiter = -(self.yaw_rate + aos) * (q + 1e5 * 0.5);
        self.add_local_moment(Vec3::new(0.0, yaw_rate_limiter, 0.0));

        let speed_limiter = limit(
            (mach.abs() / self.plane_config.basic.mach_max).powi(5) * (q + 1e5 * 0.5),
            -1e7,
            1e7,
        );
        self.add_local_force(vec3(0.0, 0.0, speed_limiter), self.center_of_mass);

        self.shake_amplitude = 0.0;

        self.shake_amplitude += limit(
            (self.plane_config.aerodynamics.cx_brk + 1.0) * self.airbrake_pos * mach,
            0.0,
            2.0,
        ) / 6.0;

        if !self.on_ground {
            if self.alpha.abs() > 10.0 {
                self.shake_amplitude += (self.alpha.abs() - 10.0) / 100.0;
            }

            if self.beta.abs() > 10.0 {
                self.shake_amplitude += (self.beta.abs() - 10.0) / 100.0;
            }

            if self.g.abs() > 5.0 {
                self.shake_amplitude += (self.g.abs() - 5.0) / 100.0;
            }

            if mach > self.plane_config.basic.mach_max * 0.8 {
                self.shake_amplitude += (mach - (self.plane_config.basic.mach_max * 0.8)) / 2.0;
            }
        }
    }

    pub fn simulate(&mut self, dt: f32, t: &Transform) {
        self.common_moment = Vec3::ZERO;
        self.common_force = Vec3::ZERO;
        self.draw_vecs = Vec::new();

        self.gear_pos = limit(
            actuator(self.gear_pos, self.gear_switch as u8 as f32, -0.001, 0.001),
            0.0,
            1.0,
        ); // Landing gear (all 3)
        self.airbrake_pos = limit(
            actuator(
                self.airbrake_pos,
                self.airbrake_switch as u8 as f32,
                -0.003,
                0.004,
            ),
            0.0,
            1.0,
        ); // Air brakes
        self.flaps_pos = limit(
            actuator(
                self.flaps_pos,
                self.flaps_switch as u8 as f32,
                -0.002,
                0.002,
            ),
            0.0,
            1.0,
        ); // Flaps
        self.slats_pos = limit(
            actuator(self.slats_pos, (self.alpha - 6.0) / 12.0, -0.003, 0.003),
            0.0,
            1.0,
        ); // Slats, starts moving at 6 degrees alpha

        // --- Aerodynamics ---
        self.airspeed = self.velocity_local;

        let mach = self.airspeed.length() / self.speed_of_sound;

        let aero = &self.plane_config.aerodynamics;
        let at = &aero.tables;

        let cy_alpha = table_lerp(&at.mach, &at.Cya, mach);
        let cx0 = table_lerp(&at.mach, &at.cx0, mach);
        let cy_max = table_lerp(&at.mach, &at.CyMax, mach) + aero.cy_flap * 0.4 * self.slats_pos;
        let alpha_max = table_lerp(&at.mach, &at.Aldop, mach);
        let omx_max = table_lerp(&at.mach, &at.OmxMax, mach);

        let cy = limit(cy_alpha * self.alpha, -cy_max, cy_max);
        let cy_tail = limit((0.5 * cy_alpha + aero.Czbe) * self.beta, -cy_max, cy_max);

        let q = 0.5
            * table_lerp(&ALTITUDE_M, &AIR_DENSITY_KG_PER_M3, self.altitude_asl)
            * self.airspeed.length_squared();

        let aos = self.beta.to_radians();
        let aoa = self.alpha.to_radians();
        let s = self.wingspan;

        // =================================================
        // AERODYNAMICS
        // =================================================

        let lift = cy + aero.Cy0 + (aero.cy_flap * self.flaps_pos);
        let drag = cx0
            + (aero.cx_brk * self.airbrake_pos)
            + (aero.cx_flap * self.flaps_pos)
            + (aero.cx_gear * self.gear_pos);

        self.update_wings(alpha_max, lift, drag, q, s, aos, aoa, t, cy_tail);

        self.update_elevator(aoa, q, &mach, t);

        self.update_roll(aos, aoa, q, t);

        self.update_yaw(aos, q, t);

        self.update_other(q, omx_max, mach, aos);

        // =================================================
        // General
        // =================================================
        self.sim_engine(&mach, &dt, t);

        let l_grav = (global_to_local(GRAV, t.rotation)) * self.current_mass;
        self.add_local_force(l_grav, self.center_of_mass);

        self.wheel_on_ground(t.rotation, &t);
    }

    pub fn update_variables(&mut self, transform: &Transform) {
        self.velocity_local = global_to_local(self.velocity, transform.rotation);

        let v_forward = -self.velocity_local.z; // Negate because forward is -Z
        let v_right = self.velocity_local.x;
        let v_up = self.velocity_local.y;

        if self.velocity.length() > 10.0 {
            self.alpha = -v_up.atan2(v_forward).to_degrees();
            self.beta = v_right.atan2(v_forward).to_degrees();
        } else {
            self.alpha = 0.0;
            self.beta = 0.0;
        }

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::ZYX);
        self.heading = yaw.to_degrees(); // Store in degrees to match alpha/beta
        self.pitch = pitch.to_degrees();
        self.roll = roll.to_degrees(); // Store in degrees to be consistent

        self.roll_rate = -self.angular_velocity.z;
        self.yaw_rate = self.angular_velocity.y;
        self.pitch_rate = self.angular_velocity.x;

        let ay = self.common_force.y / self.current_mass;
        self.g = (ay / 9.81) + 1.0;

        self.altitude_asl = transform.translation.y;
    }

    fn wheel_on_ground(&mut self, gr: Quat, transform: &Transform) {
        let wheels = self.plane_config.structure;

        let fwp = local_to_global((wheels.front_wheel), gr) + transform.translation;
        let blwp = local_to_global((wheels.back_left_wheel), gr) + transform.translation;
        let brwp = local_to_global((wheels.back_right_wheel), gr) + transform.translation;

        let ground_height = 0.0;
        let wheel_radius = 0.3; // Adjust to match your wheel size

        self.check_wheel(
            fwp,
            wheels.front_wheel,
            ground_height,
            wheel_radius,
            gr,
            1.0,
        );
        self.check_wheel(
            blwp,
            wheels.back_left_wheel,
            ground_height,
            wheel_radius,
            gr,
            1.0,
        );
        self.check_wheel(
            brwp,
            wheels.back_right_wheel,
            ground_height,
            wheel_radius,
            gr,
            1.0,
        );
    }

    fn check_wheel(
        &mut self,
        wheel_pos: Vec3,
        local_pos: Vec3,
        ground_height: f32,
        wheel_radius: f32,
        rotation: Quat,
        multi: f32,
    ) {
        let wheel_bottom = wheel_pos.y - wheel_radius;
        let penetration = ground_height - wheel_bottom;

        let damping = 200000.0;
        let spring_k = 4000000.0;

        let vel = (self.velocity);
        let damping_force = -damping * vel.y;

        if penetration > 0.0 {
            self.on_ground = true;
            let total = damping_force + spring_k * penetration;
            let upward_force = (global_to_local(Vec3::new(0.0, total, 0.0), rotation));
            self.add_local_force(upward_force * multi, local_pos);
        } else {
            self.on_ground = false;
        }
    }

    pub fn transform(&mut self, dt: f32, transform: &mut Transform) {
        // --- Linear motion ---
        let local_acceleration = self.common_force / self.current_mass;

        let global_acceleration = local_to_global(local_acceleration, transform.rotation);

        self.velocity += global_acceleration * dt;

        transform.translation += self.velocity * dt;

        // --- Angular motion ---
        let angular_acceleration = vec3(
            self.common_moment.x / self.moment_of_inertia[0],
            self.common_moment.y / self.moment_of_inertia[1],
            self.common_moment.z / self.moment_of_inertia[2],
        );

        self.angular_velocity += angular_acceleration * dt;

        let world_angular_velocity = local_to_global(self.angular_velocity, transform.rotation);

        let angle = world_angular_velocity.length() * dt;
        if angle > 0.00001 {
            let axis = world_angular_velocity.normalize();
            let delta_rot = Quat::from_axis_angle(axis, angle);

            transform.rotation = (delta_rot * transform.rotation).normalize();
        }
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

        self.pitch_input = -0.0;

        self.velocity = vec3(0.0, 0.0, -200.0);
    }
}
