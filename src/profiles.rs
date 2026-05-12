use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum, Serialize, Deserialize)]
pub enum BatteryPowerProfile {
    #[serde(rename = "power-saver")]
    PowerSaver, //Профиль максимальной автономности
    #[serde(rename = "balanced")]
    Balanced, //Профиль баланса
    #[serde(rename = "performance")]
    Performance, //Профиль максимальной производительности
}

impl BatteryPowerProfile {
    //конвертирует из Enum-a в строку
    pub fn as_str(&self) -> &str {
        match self {
            BatteryPowerProfile::PowerSaver => "power-saver",
            BatteryPowerProfile::Balanced => "balanced",
            BatteryPowerProfile::Performance => "performance",
        }
    }

    //конвертирует из строки в Enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "power-saver" => Some(Self::PowerSaver),
            "balanced" => Some(Self::Balanced),
            "performance" => Some(Self::Performance),
            _ => None,
        }
    }
}

//установка профиля
pub async fn set_profile(profile: BatteryPowerProfile) -> Result<(), std::io::Error> {
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
pub async fn get_profile() -> Result<BatteryPowerProfile, std::io::Error> {
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
