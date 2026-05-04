use std::{io, process::Command};

fn set_profile(profile: &str) -> Result<(), std::io::Error> {
    let output = Command::new("powerprofilesctl")
        .args(["set", profile])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

fn get_profile() -> Result<String, std::io::Error> {
    let output = Command::new("powerprofilesctl").arg("get").output()?;

    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(profile)
}

//износ батареи
fn get_battery_wear() -> Result<f32, std::io::Error> {
    let full = std::fs::read_to_string("/sys/class/power_supply/BAT0/energy_full")?;
    let design = std::fs::read_to_string("/sys/class/power_supply/BAT0/energy_full_design")?;

    let full: f32 = full
        .trim()
        .parse()
        .map_err(|_| io::Error::other("parse error: energy_full"))?;

    let design: f32 = design
        .trim()
        .parse()
        .map_err(|_| io::Error::other("parse error: energy_full_design"))?;

    Ok((full / design) * 100.0)
}

fn get_status() -> Result<String, std::io::Error> {
    let status = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")?;
    Ok(status.trim().to_string())
}

fn print_battery_info() {
    match get_profile() {
        Ok(n) => println!("На данных момент стоит профиль: {}", n),
        Err(e) => println!("Ошибка получения профиля батареи: {}", e),
    }

    match get_status() {
        Ok(n) => println!("Статус: {}", n),
        Err(e) => println!("Ошибка получения статуса батареи: {}", e),
    }

    match get_battery_wear() {
        Ok(n) => println!("Износ батареи: {}", n.trunc()),
        Err(e) => println!("Ошибка получения износа батареи: {}", e),
    }
}

fn main() -> Result<(), std::io::Error> {
    println!("==========");
    println!("Информация о системе\n==========");

    print_battery_info();

    println!("==========");

    println!("Введите номер профиля:\n\t0 - power-saver\n\t1 - balanced\n\t2 - performance");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let input: usize = match input.trim().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Ошибка входящей информации: введите число");
            return Ok(());
        }
    };

    let power = ["power-saver", "balanced", "performance"];

    match power.get(input) {
        Some(profile) => {
            set_profile(profile)?;
            println!("Вы установили профиль: {}", profile);
        }
        None => println!("Ошибка индекса: неверный индекс"),
    }
    /*
    if let Some(profile) = power.get(input) {
        match set_profile(profile) {
            Ok(_) => println!("Вы установили профиль: {}", profile),
            Err(e) => println!("Ошибка Установки профиля: {}", e),
        }
    } else {
        println!("Ошибка Индекса: Неверный индекс");
    }
    */

    Ok(())
}
