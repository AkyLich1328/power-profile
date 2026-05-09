////////////////////////////////////
use clap::ValueEnum;
use clap::{Parser, Subcommand};
use std::{io, path::PathBuf};
use tokio::time::{Duration, sleep};
////////////////////////////////////

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
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum BatteryPowerProfile {
    PowerSaver,  //Профиль максимальной автономности
    Balanced,    //Профиль баланса
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

async fn get_battery_info() -> Result<BatteryInfo, std::io::Error> {
    Ok(BatteryInfo {
        profile: get_profile().await?,
        status: get_status().await?,
        wear: get_battery_wear().await?,
        capacity: get_capacity().await?,
    })
}

async fn print_battery_info() {
    match get_battery_info().await {
        Ok(info) => {
            println!("Профиль: {}", info.profile.as_str());
            println!("Статус: {}", info.status);
            println!("Заряд: {}%", info.capacity);
            println!("Износ: {:.2}%", info.wear);
        }

        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }
}

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

//Главный поток
async fn auto_profile_worker() {
    println!("Начало работы автоматической системы профилей батареи...");
    //Получение температуры процессора по термальной зоне
    let cpu_temp_path = match find_cpu_temp_path() {
        Ok(path) => {
            println!("Термальная зона процессора найдена: {}", path.display());
            Some(path)
        }
        Err(e) => {
            println!("Не удалось найти x86_pkg_temp: {}", e);
            None
        }
    };

    loop {
        tokio::select! {
           _ = tokio::signal::ctrl_c() => {
               println!("\nЗавершение daemon...");
               break;
           }

            //Делаем каждые 10 секунд проверки и смену профиля
            _ = sleep(Duration::from_secs(10)) => {
                let status = match get_status().await {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Ошибка статуса: {}", e);
                        continue;
                    }
                };

                let capacity = match get_capacity().await {
                    Ok(c) => c,
                    Err(e) => {
                        println!("Ошибка заряда: {}", e);
                        continue;
                    }
                };

                // Читаем температуру CPU, если удалось найти x86_pkg_temp
                let cpu_temp = match &cpu_temp_path {
                    Some(path) => match get_cpu_temperature(path).await {
                        Ok(temp) => Some(temp),
                        Err(e) => {
                            println!("Ошибка чтения температуры CPU: {}", e);
                            None
                        }
                    },
                    None => None,
                };

                let target_profile =
                    //если температура процессора больше или равна 85% или заряд батареи
                    //меньше или равен 25 то ставим профиль PowerSaver
                    if cpu_temp.is_some_and(|temp| temp >= 85.0)
                        || capacity <= 25 {
                        BatteryPowerProfile::PowerSaver
                    } else {
                        match status {
                        //если работает от зарядки
                        BatteryChargeStatus::Charging => BatteryPowerProfile::Performance,
                        //если работает от батареи
                        BatteryChargeStatus::Discharging => BatteryPowerProfile::Balanced,
                        //если батарея заряженна полностью
                        BatteryChargeStatus::Full => BatteryPowerProfile::Performance,
                        //Если неисзвестно состояние батареи
                        //забавное наблюдение, когда стоит ограничение на зарядку 95%
                        //то когда батарея будет на этом уровне зарядки то будет Unknown
                        BatteryChargeStatus::Unknown => BatteryPowerProfile::Balanced,
                    }
                };

                //Получение текущего профиля
                let current_profile = match get_profile().await {
                    Ok(profile) => profile,

                    Err(e) => {
                        println!("Ошибка получения профиля: {}", e);

                        continue;
                    }
                };


                if current_profile != target_profile {
                    match set_profile(target_profile).await {
                        Ok(_) => {
                            println!("Установлен профиль: {}", target_profile.as_str());
                        }

                        Err(e) => {
                            println!("Ошибка установки: {}", e);
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
            println!("Установлен профиль: {}", profile.as_str());
        }
    }

    println!("==========");

    Ok(())
}
