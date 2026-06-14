use std::{fs, io, path::PathBuf};

use crate::logging::log_msg;
/// Ищет thermal zone с типом x86_pkg_temp
pub fn find_cpu_temp_path() -> Result<PathBuf, io::Error> {
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
pub async fn get_cpu_temperature(temp_path: &std::path::Path) -> Result<f32, io::Error> {
    let temp = tokio::fs::read_to_string(temp_path).await?;

    let temp: f32 = temp
        .trim()
        .parse()
        .map_err(|_| io::Error::other("Ошибка парсинга: cpu temp"))?;

    Ok(temp / 1000.0)
}

pub fn set_intel_turbo_bost(disable: bool) -> Result<(), std::io::Error> {
    let path = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    let value: u8 = fs::read_to_string(path)
        .expect("Не удалось прочитать файл no_turbo")
        .trim()
        .parse()
        .expect("В файле должно быть число");

    match value {
        1 => {
            if !disable {
                match fs::write(path, "0") {
                    Ok(()) => log_msg("Успешно включаем TurboBoost"),
                    Err(e) => println!("Ошибка TurboBoost: {}", e),
                }
            }
        }
        0 => {
            if disable {
                match fs::write(path, "1") {
                    Ok(()) => log_msg("Успешно отключаем TurboBoost"),
                    Err(e) => println!("Ошибка TurboBoost: {} ", e),
                }
            }
        }
        _ => {}
    }

    Ok(())
}
