mod auto;
mod battery;
mod cli;
mod config;
mod logging;
mod models;
mod profiles;
mod thermal;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => {
            battery::print_battery_info().await;
        }

        Commands::Auto => {
            auto::auto_profile_worker().await;
        }

        Commands::Set { profile } => {
            profiles::set_profile(profile).await?;
            logging::log_msg(format!("Установлен профиль: {}", profile.as_str()));
        }
    }

    Ok(())
}
