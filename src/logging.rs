use std::fmt::Display;
pub fn log_msg(message: impl Display) {
    let now = chrono::Local::now();
    println!(
        "[{}] Power-Profile - {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        message
    );
}
