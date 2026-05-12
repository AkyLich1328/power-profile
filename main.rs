////////////////////////////////////
use clap::ValueEnum;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::{io, path::PathBuf};
use tokio::time::{Duration, sleep};
////////////////////////////////////

//Для конфига
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    low_battery_threshold: u8,
    high_temperature_threshold: f32,
    charging_profile: BatteryPowerProfile,
    battery_profile: BatteryPowerProfile,
    enable_charge_limit: bool,
    charge_limit: u8,
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

fn load_config(path: &str) -> Result<Config, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents).map_err(io::Error::other)?;

    Ok(config)
}

fn save_config(path: &str, config: &Config) -> Result<(), io::Error> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;

    fs::write(path, json)?;
    Ok(())
}

fn load_or_create_config(path: &str) -> Result<Config, io::Error> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            let config = Config::default();
            save_config(path, &config)?;
            Ok(config)
        }
    }
}

//Для удобного управления аргументами в терминале
#[derive(Parser)]
#[command(name = "power-profile")]
#[command(version = "1.0")]
#[command(about = "Linux power manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Info,
    Auto,
    Set { profile: BatteryPowerProfile },
}
/////////////////////////////////////////////

//Для профилей
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum, Serialize, Deserialize)]
enum BatteryPowerProfile {
    #[serde(rename = "power-saver")]
    PowerSaver, //Профиль максимальной автономности
    #[serde(rename = "balanced")]
    Balanced, //Профиль баланса
    #[serde(rename = "performance")]
    Performance, //Профиль максимальной производительности
}

impl BatteryPowerProfile {
    //конвертирует из Enum-a в строку
    fn as_str(&self) -> &str {
        match self {
            BatteryPowerProfile::PowerSaver => "power-saver",
            BatteryPowerProfile::Balanced => "balanced",
            BatteryPowerProfile::Performance => "performance",
        }
    }

    //конвертирует из строки в Enum
    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "power-saver" => Some(Self::PowerSaver),
            "balanced" => Some(Self::Balanced),
            "performance" => Some(Self::Performance),
            _ => None,
        }
    }
}
////////////////////////////////

//Для статуса зарядки
enum BatteryChargeStatus {
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
    fn from_str(s: &str) -> Self {
        match s.trim() {
            "Charging" => Self::Charging,
            "Discharging" => Self::Discharging,
            "Full" => Self::Full,
            _ => Self::Unknown,
        }
    }
}
///////////////////////////////////////////////////

//Информация о батареи
struct BatteryInfo {
    profile: BatteryPowerProfile, //Профиль
    status: BatteryChargeStatus,  //Статус зарядки
    wear: f32,                    //Износ в процентах
    capacity: u8,                 //Уровень зарядки в процентах
}

//установка профиля
async fn set_profile(profile: BatteryPowerProfile) -> Result<(), std::io::Error> {
    let output = tokio::process::Command::new("powerprofilesctl")
        .args(["set", profile.as_str()])
        .output()
        .await?;

    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

//полцчение текущего профиля
async fn get_profile() -> Result<BatteryPowerProfile, std::io::Error> {
    let output = tokio::process::Command::new("powerprofilesctl")
        .arg("get")
        .output()
        .await?;

    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let profile_str = String::from_utf8_lossy(&output.stdout);

    BatteryPowerProfile::from_str(&profile_str).ok_or(std::io::Error::other("Неизвестный профиль"))
}

/////////////////////////////////////////

//Функция поиска директории которая отвечает за информацию о батареи
fn find_battery_path() -> Result<PathBuf, io::Error> {
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
async fn get_battery_wear() -> Result<f32, io::Error> {
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
async fn get_status() -> Result<BatteryChargeStatus, std::io::Error> {
    let baterry_path = find_battery_path()?;
    let status = tokio::fs::read_to_string(baterry_path.join("status")).await?;

    Ok(BatteryChargeStatus::from_str(&status))
}

//количество зарядки
async fn get_capacity() -> Result<u8, std::io::Error> {
    let baterry_path = find_battery_path()?;
    let capacity = tokio::fs::read_to_string(baterry_path.join("capacity")).await?;

    let capacity: u8 = capacity
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка: не удалось спарсить capacity"))?;

    Ok(capacity)
}

//Получение информации о батареи
async fn get_battery_info() -> Result<BatteryInfo, std::io::Error> {
    Ok(BatteryInfo {
        profile: get_profile().await?,
        status: get_status().await?,
        wear: get_battery_wear().await?,
        capacity: get_capacity().await?,
    })
}

//Вывод информации о батареи
async fn print_battery_info() {
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
fn set_charge_limit(limit: u8) -> Result<(), std::io::Error> {
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

///////////////////////////////////////////
/// Ищет thermal zone с типом x86_pkg_temp
fn find_cpu_temp_path() -> Result<PathBuf, io::Error> {
    for entry in std::fs::read_dir("/sys/class/thermal")? {
        let entry = entry?;
        let path = entry.path();

        let Some(name) = path.file_name() else {
            continue;
        };

        if !name.to_string_lossy().starts_with("thermal_zone") {
            continue;
        }

        let zone_type = std::fs::read_to_string(path.join("type"))?;

        if zone_type.trim() == "x86_pkg_temp" {
            return Ok(path.join("temp"));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Ошибка: x86_pkg_temp не нашелся",
    ))
}

/// Читает температуру CPU в градусах
async fn get_cpu_temperature(temp_path: &std::path::Path) -> Result<f32, io::Error> {
    let temp = tokio::fs::read_to_string(temp_path).await?;

    let temp: f32 = temp
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка парсинга: cpu temp"))?;

    Ok(temp / 1000.0)
}
/////////////////////////////////////

//Функция логирования
fn log_msg(message: impl Display) {
    let now = chrono::Local::now();
    println!(
        "[{}] Power-Profile - {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        message
    );
}

//Главный поток
async fn auto_profile_worker() {
    println!("Начало работы автоматической системы профилей батареи...");
    //Получение температуры процессора по термальной зоне
    let cpu_temp_path = match find_cpu_temp_path() {
        Ok(path) => {
            log_msg(format!(
                "Термальная зона процессора найдена: {}",
                path.display()
            ));
            Some(path)
        }
        Err(e) => {
            log_msg(format!("Не удалось найти x86_pkg_temp: {}", e));
            None
        }
    };

    let config = match load_or_create_config("/etc/power-profile/config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            log_msg(format!("Ошибка чтения конфига: {}", e));
            Config::default()
        }
    };

    if config.enable_charge_limit {
        match set_charge_limit(config.charge_limit) {
            Ok(()) => {
                log_msg(format!(
                    "Установлен лимит зарядки батареи: {}",
                    config.charge_limit
                ));
            }
            Err(e) => {
                log_msg(format!("Ошибка установки лимита зарядки: {}", e));
            }
        }
    } else {
        log_msg("Лимит заряда батареи: Выключен");
    }

    loop {
        tokio::select! {
           _ = tokio::signal::ctrl_c() => {
                log_msg("Завершение работы");
               break;
           }

            //Делаем каждые 10 секунд проверки и смену профиля
            _ = sleep(Duration::from_secs(10)) => {
                let status = match get_status().await {
                    Ok(s) => s,
                    Err(e) => {
                        log_msg(format!("Ошибка статуса: {}", e));
                        continue;
                    }
                };

                let capacity = match get_capacity().await {
                    Ok(c) => c,
                    Err(e) => {
                        log_msg(format!("Ошибка заряда: {}", e));
                        continue;
                    }
                };

                // Читаем температуру CPU, если удалось найти x86_pkg_temp
                let cpu_temp = match &cpu_temp_path {
                    Some(path) => match get_cpu_temperature(path).await {
                        Ok(temp) => Some(temp),
                        Err(e) => {
                            log_msg(format!("Ошибка чтения температуры CPU: {}", e));
                            None
                        }
                    },
                    None => None,
                };

                let (target_profile, reason) =
                    //если температура процессора больше или равна 85% или заряд батареи
                    //меньше или равен 25 то ставим профиль PowerSaver
                    if cpu_temp.is_some_and(|temp| temp >= config.high_temperature_threshold) || capacity <= config.low_battery_threshold {
                        (
                            BatteryPowerProfile::PowerSaver,
                            "Температура CPU => 85°C или заряд батареи <= 25%",
                        )
                    } else {
                        match status {
                        //если работает от зарядки
                        BatteryChargeStatus::Charging => (
                            config.charging_profile,
                            "Работает от зарядки"
                        ),
                        //если работает от батареи
                        BatteryChargeStatus::Discharging => (
                            config.battery_profile,
                                "Работает от батареи",
                        ),
                        //если батарея заряженна полностью
                        BatteryChargeStatus::Full => (
                            BatteryPowerProfile::Performance,
                                "Батарея заряженна полностью",
                        ),
                        //Если неисзвестно состояние батареи
                        //забавное наблюдение, когда стоит ограничение на зарядку 95%
                        //то когда батарея будет на этом уровне зарядки то будет Unknown
                        BatteryChargeStatus::Unknown => (
                            BatteryPowerProfile::Balanced,
                                "Неизсветное состояние батареи",
                        ),
                    }
                };

                //Получение текущего профиля
                let current_profile = match get_profile().await {
                    Ok(profile) => profile,

                    Err(e) => {
                        log_msg(format!("Ошибка получения профиля: {}", e));

                        continue;
                    }
                };


                if current_profile != target_profile {

                    log_msg(format!(
                        "Смена профиля: {} -> {} ({})",
                        current_profile.as_str(),
                        target_profile.as_str(),
                        reason
                    ));

                    match set_profile(target_profile).await {
                        Ok(_) => {
                            log_msg(format!("Установлен профиль: {}", target_profile.as_str()));
                        }

                        Err(e) => {
                            log_msg(format!("Ошибка установки: {}", e));
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => {
            print_battery_info().await;
        }

        Commands::Auto => {
            auto_profile_worker().await;
        }

        Commands::Set { profile } => {
            set_profile(profile).await?;
            log_msg(format!("Установлен профиль: {}", profile.as_str()));
        }
    }

    Ok(())
}
