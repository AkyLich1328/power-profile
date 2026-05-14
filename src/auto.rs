use tokio::time::{Duration, sleep};

use crate::battery::BatteryChargeStatus;
use crate::battery::get_capacity;
use crate::battery::get_status;
use crate::battery::set_charge_limit;
use crate::config::Config;
use crate::config::load_or_create_config;
use crate::logging::log_msg;
use crate::profiles::BatteryPowerProfile;
use crate::profiles::get_profile;
use crate::profiles::set_profile;
use crate::thermal::find_cpu_temp_path;
use crate::thermal::get_cpu_temperature;

pub async fn auto_profile_worker() {
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
                            config.full_battery_profile,
                                "Батарея заряженна полностью",
                        ),
                        //Если неисзвестно состояние батареи
                        //забавное наблюдение, когда стоит ограничение на зарядку 95%
                        //то когда батарея будет на этом уровне зарядки то будет Unknown
                        BatteryChargeStatus::Unknown => (
                            config.unknown_profile,
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
