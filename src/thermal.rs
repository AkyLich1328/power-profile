use std::{io, path::PathBuf};
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
