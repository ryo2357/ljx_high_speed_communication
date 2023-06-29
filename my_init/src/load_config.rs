use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

pub static CONFIG: Lazy<AppConfig> = Lazy::new(set_config);

const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub save_dir: String,
    pub ljx_profile_data_num: usize,
    pub ljx_fetch_brightness_data: bool,
    pub ljx_ip_address: [u8; 4],
    pub ljx_port: u16,
    pub ljx_high_speed_port: u16,
}
// フィールドもpub にしないとアクセスできない

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            save_dir: "./output".to_string(),
            ljx_profile_data_num: 3200,
            ljx_fetch_brightness_data: true,
            ljx_ip_address: [192, 168, 0, 1],
            ljx_port: 24691,
            ljx_high_speed_port: 24692,
        }
    }
}
fn set_config() -> AppConfig {
    let path: String = "config/".to_string() + NAME + ".toml";
    match confy::load_path::<AppConfig>(path) {
        Ok(v) => v,
        Err(_e) => AppConfig::default(),
    }
}
