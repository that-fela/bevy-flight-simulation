#![allow(dead_code)]
#![allow(non_snake_case)]

use bevy::math::Vec3;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Unit {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Basic {
    pub wing_area: f32,
    pub wingspan: f32,
    pub length: f32,
    pub height: f32,
    pub mach_max: f32,
    pub center_of_mass: [f32; 3],
    pub moment_of_inertia: [f32; 4],
    pub empty_mass: f32,
    pub gross_mass: f32,
    pub number_of_engines: u8,
}

#[derive(Debug, Deserialize)]
pub struct Aerodynamics {
    pub Cy0: f32,
    pub Czbe: f32,
    pub cx_gear: f32,
    pub cx_brk: f32,
    pub cx_flap: f32,
    pub cy_flap: f32,
    pub tables: AeroTables,
}

#[derive(Debug, Deserialize)]
pub struct AeroTables {
    pub mach: Vec<f32>,
    pub cx0: Vec<f32>,
    pub Cya: Vec<f32>,
    pub OmxMax: Vec<f32>,
    pub Aldop: Vec<f32>,
    pub CyMax: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct Engine {
    pub idle_rpm: f32,
    pub fuel_consumption: f32,
    pub engine_start_time: f32,
    pub tables: EngineTables,
}

#[derive(Debug, Deserialize)]
pub struct EngineTables {
    pub mach: Vec<f32>,
    pub max_thrust: Vec<f32>,
    pub throttle_input: Vec<f32>,
    pub engine_power: Vec<f32>,
    pub engine_power_readout: Vec<f32>,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Structure {
    pub front_wheel: Vec3,
    pub back_left_wheel: Vec3,
    pub back_right_wheel: Vec3,
    pub left_wing_pos: Vec3,
    pub right_wing_pos: Vec3,
    pub tail_pos: Vec3,
    pub elevator_pos: Vec3,
    pub left_aileron_pos: Vec3,
    pub right_aileron_pos: Vec3,
    pub rudder_pos: Vec3,
    pub left_engine_pos: Vec3,
    pub right_engine_pos: Vec3,
}

/// Main configuration struct
#[derive(Debug, Deserialize)]
pub struct PlaneConfig {
    pub unit: Unit,
    pub basic: Basic,
    pub aerodynamics: Aerodynamics,
    pub engine: Engine,
    pub structure: Structure,
}

pub fn load_config(plane_type: &str) -> PlaneConfig {
    let config_path = format!("assets/aircraft/{}/config.toml", plane_type);
    let config_str = std::fs::read_to_string(config_path).expect("Failed to read config file");
    let config: PlaneConfig = toml::from_str(&config_str).expect("Failed to parse config file");
    config
}

impl PlaneConfig {
    pub fn new(plane_type: &str) -> Self {
        load_config(plane_type)
    }
}

impl Default for PlaneConfig {
    fn default() -> Self {
        PlaneConfig::new("su-25t")
    }
}
