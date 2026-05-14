use crate::profiles::BatteryPowerProfile;
use serde::{Deserialize, Serialize};
use std::{fs, io};

//Основа конфиг файла
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub low_battery_threshold: u8, //процент батареи после которого идет экономия заряда
    pub high_temperature_threshold: f32, //температура после которой идет экономия заряда
    pub charging_profile: BatteryPowerProfile, //профиль питания при зарядке
    pub full_battery_profile: BatteryPowerProfile, //профиль питания при полном заряде батареи
    pub battery_profile: BatteryPowerProfile, //профиль при работе от батареи
    pub unknown_profile: BatteryPowerProfile, //профиль при неизвестном состоянии батареи(обычно
    //неизвестное состояние батареи тогда когда стоит лимит на зарядку и батарея заряженна на этом уровне
    pub enable_charge_limit: bool, //Установка лимита зарядки
    pub charge_limit: u8,          //процент после которого прекратится заряжаться батарея
}

impl Default for Config {
    //Стандартные настройки конфига
    fn default() -> Self {
        Self {
            low_battery_threshold: 25,
            high_temperature_threshold: 85.0,
            charging_profile: BatteryPowerProfile::Performance,
            full_battery_profile: BatteryPowerProfile::Performance,
            battery_profile: BatteryPowerProfile::PowerSaver,
            unknown_profile: BatteryPowerProfile::Balanced,
            enable_charge_limit: false,
            charge_limit: 85,
        }
    }
}

//Загрузка конфиг файла
pub fn load_config(path: &str) -> Result<Config, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents).map_err(io::Error::other)?;

    Ok(config)
}

//Сохранение конфиг файла
pub fn save_config(path: &str, config: &Config) -> Result<(), io::Error> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;

    fs::write(path, json)?;
    Ok(())
}

//Загрузка конфиг файла или его создание если его нет
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
