//Информация о батареи
use crate::battery::BatteryChargeStatus;
use crate::profiles::BatteryPowerProfile;

pub struct BatteryInfo {
    pub profile: BatteryPowerProfile, //Профиль
    pub status: BatteryChargeStatus,  //Статус зарядки
    pub wear: f32,                    //Износ в процентах
    pub capacity: u8,                 //Уровень зарядки в процентах
}
