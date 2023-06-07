use chrono::{DateTime, Local};
use std::io::Write;

pub fn get_time_string() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d_%H%M%S").to_string()
}

pub fn wait_until_enter() {
    print!("wait until press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
