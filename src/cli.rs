use crate::profiles::BatteryPowerProfile;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(name = "power-profile")]
#[command(version = "1.0")]
#[command(about = "Linux power manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Info,
    Auto,
    Set { profile: BatteryPowerProfile },
}
