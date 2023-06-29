use dotenv::dotenv;
use serde::Deserialize;
// use std::env;

use ljx_high_speed_communication::profile_converter::ProfileConverterConfig;

fn main() {
    dotenv().ok();

    let mut config =
        match envy::prefixed("ProfileConverterConfig_").from_env::<ProfileConverterConfig>() {
            Ok(config) => config,

            Err(error) => panic!("{:#?}", error),
        };

    println!("{:#?}", config);
    println!("{:#?}", config.check_completeness());

    config.set_profile_path("test".to_string());

    println!("{:#?}", config);
    println!("{:#?}", config.check_completeness());
}
