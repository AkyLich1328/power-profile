use crate::profiles::BatteryPowerProfile;
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub low_battery_threshold: u8,
    pub high_temperature_threshold: f32,
    pub charging_profile: BatteryPowerProfile,
    pub battery_profile: BatteryPowerProfile,
    pub enable_charge_limit: bool,
    pub charge_limit: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            low_battery_threshold: 25,
            high_temperature_threshold: 85.0,
            charging_profile: BatteryPowerProfile::Performance,
            battery_profile: BatteryPowerProfile::PowerSaver,
            enable_charge_limit: false,
            charge_limit: 85,
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents).map_err(io::Error::other)?;

    Ok(config)
}

pub fn save_config(path: &str, config: &Config) -> Result<(), io::Error> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;

    fs::write(path, json)?;
    Ok(())
}

pub fn load_or_create_config(path: &str) -> Result<Config, io::Error> {
    match load_config(path) {
        /////////////////////////////////////////////
        Ok(config) => Ok(config),
        Err(_) => {
            let config = Config::default();
            save_config(path, &config)?;
            Ok(config)
        }
    }
}
