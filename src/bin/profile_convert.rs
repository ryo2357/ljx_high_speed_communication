use dotenv::dotenv;
use log::info;

use ljx_high_speed_communication::converter_ply::{
    convert_ljx_data_to_ply, LjxDataConverterConfig,
};

fn main() -> anyhow::Result<()> {
    my_init::init();
    info!("logger initialized");

    dotenv().ok();
    let config = envy::prefixed("LjxDataConverterConfig_").from_env::<LjxDataConverterConfig>()?;

    convert_ljx_data_to_ply(config)?;

    Ok(())
}
