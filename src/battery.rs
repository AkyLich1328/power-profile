use crate::logging::log_msg;
use crate::models::BatteryInfo;
use crate::profiles::get_profile;
use std::{io, path::PathBuf};
//Для статуса зарядки
pub enum BatteryChargeStatus {
    Charging,    //Работа от зарядки
    Discharging, //Работа от батареи
    Full,        //Батарея заряженна на максимум
    Unknown,     //Неизвестно
}

//Для удобного конвертирования в строку
impl std::fmt::Display for BatteryChargeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Charging => "Charging",
            Self::Discharging => "Discharging",
            Self::Full => "Full",
            Self::Unknown => "Unknown",
        };

        write!(f, "{}", s)
    }
}

impl BatteryChargeStatus {
    //Конверитрование строки в Enum
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "Charging" => Self::Charging,
            "Discharging" => Self::Discharging,
            "Full" => Self::Full,
            _ => Self::Unknown,
        }
    }
}
//////////////////////////////////////////////////

//Функция поиска директории которая отвечает за информацию о батареи
pub fn find_battery_path() -> Result<PathBuf, io::Error> {
    for entry in std::fs::read_dir("/sys/class/power_supply")? {
        let entry = entry?;
        let path = entry.path();

        let type_path = path.join("type");

        if !type_path.exists() {
            continue;
        }

        let power_type = std::fs::read_to_string(type_path)?;

        if power_type.trim() == "Battery" {
            return Ok(path);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Ошибка: Батарея не была найдена",
    ))
}

//износ батареи
//на разных ноутбуках стоит energy или charge, по этому тут ищем то что подходит
pub async fn get_battery_wear() -> Result<f32, io::Error> {
    let battery_path = find_battery_path()?;

    let full_path = if battery_path.join("energy_full").exists() {
        battery_path.join("energy_full")
    } else {
        battery_path.join("charge_full")
    };

    let design_path = if battery_path.join("energy_full_design").exists() {
        battery_path.join("energy_full_design")
    } else {
        battery_path.join("charge_full_design")
    };

    let full: f32 = tokio::fs::read_to_string(full_path)
        .await?
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка парсинга: full_path)"))?;

    let design: f32 = tokio::fs::read_to_string(design_path)
        .await?
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка парсинга: full_design"))?;

    Ok((full / design) * 100.0)
}

//статус зарядки
pub async fn get_status() -> Result<BatteryChargeStatus, std::io::Error> {
    let baterry_path = find_battery_path()?;
    let status = tokio::fs::read_to_string(baterry_path.join("status")).await?;

    Ok(BatteryChargeStatus::from_str(&status))
}

//количество зарядки
pub async fn get_capacity() -> Result<u8, std::io::Error> {
    let baterry_path = find_battery_path()?;
    let capacity = tokio::fs::read_to_string(baterry_path.join("capacity")).await?;

    let capacity: u8 = capacity
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка: не удалось спарсить capacity"))?;

    Ok(capacity)
}

//Получение информации о батареи
pub async fn get_battery_info() -> Result<BatteryInfo, std::io::Error> {
    Ok(BatteryInfo {
        profile: get_profile().await?,
        status: get_status().await?,
        wear: get_battery_wear().await?,
        capacity: get_capacity().await?,
    })
}

//Вывод информации о батареи
pub async fn print_battery_info() {
    match get_battery_info().await {
        Ok(info) => {
            log_msg(format!(
                "Профиль: {}\nСтатус: {}\nЗаряд: {}%\nИзнос: {:.2}%",
                info.profile.as_str(),
                info.status,
                info.capacity,
                info.wear
            ));
        }

        Err(e) => {
            log_msg(format!("Ошибка: {}", e));
        }
    }
}

//установка лимита зарядки
pub fn set_charge_limit(limit: u8) -> Result<(), std::io::Error> {
    if !(50..=100).contains(&limit) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Лимит должен быть в диапазоне от 50 до 100",
        ));
    }

    let battery_path = find_battery_path()?;

    let candidates = [
        "charge_control_end_threshold", // ThinkPad, ASUS, Dell
        "charge_stop_threshold",        // Некоторые ASUS
    ];

    for file in candidates {
        let path = battery_path.join(file);

        if path.exists() {
            std::fs::write(&path, limit.to_string())?;
            return Ok(());
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Этот ноутбук не поддерживает лимит зарядки",
    ))
}
