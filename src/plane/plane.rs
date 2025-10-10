use crate::plane::flight_model::FlightModel;
use crate::plane::plane_config::*;
use crate::util::*;
use bevy::prelude::*;

pub struct Plane {
    pub flight_model: FlightModel,
}

impl Plane {
    pub fn new(plane_type: &str) -> Self {
        let plane_config = load_config(plane_type);
        let mut flight_model = FlightModel::new(plane_config);
        flight_model.start_hot();

        Plane {
            flight_model: flight_model,
        }
    }

    pub fn simulate(&mut self, dt: f32, transform: &mut Transform) {
        self.flight_model.simulate(dt);
        self.flight_model.transform(dt, transform);
        self.flight_model.update_variables();
    }

    pub fn input(&mut self, keyboard: &Res<ButtonInput<KeyCode>>) {
        const TRIM_PITCH_STEP: f32 = 0.0015;
        const TRIM_ROLL_STEP: f32 = 0.001;
        const TRIM_YAW_STEP: f32 = 0.001;

        const THROTTLE_STEP: f32 = 0.0075;

        // const INPUT_MIN: f32 = -1.0;
        // const INPUT_MAX: f32 = 1.0;
        const THROTTLE_MIN: f32 = 0.0;
        const THROTTLE_MAX: f32 = 1.0;

        let controls = &mut self.flight_model;

        if keyboard.pressed(KeyCode::ArrowUp) {
            controls.pitch_discrete = 1;
            controls.pitch_analog = false;
        } else if keyboard.pressed(KeyCode::ArrowDown) {
            controls.pitch_discrete = -1;
            controls.pitch_analog = false;
        } else {
            controls.pitch_discrete = 0;
        }

        if keyboard.just_pressed(KeyCode::PageUp) {
            controls.pitch_trim += TRIM_PITCH_STEP;
        } else if keyboard.just_pressed(KeyCode::PageDown) {
            controls.pitch_trim -= TRIM_PITCH_STEP;
        }

        // --- Roll ---
        if keyboard.pressed(KeyCode::ArrowLeft) {
            controls.roll_discrete = -1;
            controls.roll_analog = false;
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            controls.roll_discrete = 1;
            controls.roll_analog = false;
        } else {
            controls.roll_discrete = 0;
        }

        if keyboard.just_pressed(KeyCode::KeyQ) {
            controls.roll_trim -= TRIM_ROLL_STEP;
        } else if keyboard.just_pressed(KeyCode::KeyE) {
            controls.roll_trim += TRIM_ROLL_STEP;
        }

        // --- Yaw ---
        if keyboard.pressed(KeyCode::KeyZ) {
            controls.yaw_discrete = 1;
            controls.yaw_analog = false;
        } else if keyboard.pressed(KeyCode::KeyX) {
            controls.yaw_discrete = -1;
            controls.yaw_analog = false;
        } else {
            controls.yaw_discrete = 0;
        }

        if keyboard.just_pressed(KeyCode::Comma) {
            controls.yaw_trim += TRIM_YAW_STEP;
        } else if keyboard.just_pressed(KeyCode::Period) {
            controls.yaw_trim -= TRIM_YAW_STEP;
        }

        // --- Reset trim ---
        if keyboard.just_pressed(KeyCode::KeyR) {
            controls.pitch_trim = 0.0;
            controls.roll_trim = 0.0;
            controls.yaw_trim = 0.0;
        }

        // --- Throttle ---
        if keyboard.pressed(KeyCode::KeyW) {
            controls.left_throttle_input = limit(
                controls.left_throttle_input + THROTTLE_STEP,
                THROTTLE_MIN,
                THROTTLE_MAX,
            );
            controls.right_throttle_input = limit(
                controls.right_throttle_input + THROTTLE_STEP,
                THROTTLE_MIN,
                THROTTLE_MAX,
            );
        } else if keyboard.pressed(KeyCode::KeyS) {
            controls.left_throttle_input = limit(
                controls.left_throttle_input - THROTTLE_STEP,
                THROTTLE_MIN,
                THROTTLE_MAX,
            );
            controls.right_throttle_input = limit(
                controls.right_throttle_input - THROTTLE_STEP,
                THROTTLE_MIN,
                THROTTLE_MAX,
            );
        }

        // --- Engine toggles ---
        if keyboard.just_pressed(KeyCode::Digit1) {
            controls.left_engine_switch = !controls.left_engine_switch;
        }
        if keyboard.just_pressed(KeyCode::Digit2) {
            controls.right_engine_switch = !controls.right_engine_switch;
        }

        // --- Airbrake ---
        if keyboard.just_pressed(KeyCode::KeyB) {
            controls.airbrake_switch = !controls.airbrake_switch;
        }

        // --- Flaps ---
        if keyboard.just_pressed(KeyCode::KeyF) {
            controls.flaps_switch = !controls.flaps_switch;
        }

        // --- Gear ---
        if keyboard.just_pressed(KeyCode::KeyG) {
            controls.gear_switch = !controls.gear_switch;
        }

        // --- Wheel brakes ---
        if keyboard.pressed(KeyCode::KeyT) {
            controls.wheel_brake = 1.0;
        } else {
            controls.wheel_brake = 0.0;
        }
    }

    pub fn start_hot(&mut self) {
        self.flight_model.start_hot();
    }
}
