use crate::logging::log_msg;

pub async fn set_brightness(percent: u8) -> Result<(), std::io::Error> {
    let percent = percent.min(100);

    let max_brightness: u32 =
        tokio::fs::read_to_string("/sys/class/backlight/intel_backlight/max_brightness")
            .await?
            .trim()
            .parse::<u32>()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Ошибка чтения max_brightness",
                )
            })?;

    let brightness = max_brightness * percent as u32 / 100;

    tokio::fs::write(
        "/sys/class/backlight/intel_backlight/brightness",
        brightness.to_string(),
    )
    .await
    .map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Ошибка чтения brightness")
    })?;

    log_msg(format!("Успешно меняем яркость экрана на {}%", percent));
    Ok(())
}
