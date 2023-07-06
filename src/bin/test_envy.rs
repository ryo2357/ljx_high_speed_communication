use dotenv::dotenv;
// use std::env;

use ljx_high_speed_communication::converter_raw_to_image::LjxDataConverterConfig;

fn main() {
    dotenv().ok();

    let mut config =
        match envy::prefixed("LjxDataConverterConfig").from_env::<LjxDataConverterConfig>() {
            Ok(config) => config,

            Err(error) => panic!("{:#?}", error),
        };

    println!("{:#?}", config);
    println!("{:#?}", config.check_completeness());

    config.set_ljx_data_path("test".to_string());

    println!("{:#?}", config);
    println!("{:#?}", config.check_completeness());
}
