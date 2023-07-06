//  高速データ通信、バッチ測定

use dotenv::dotenv;
use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;

use ljx::LjxIf;
use log::info;

// TODO:そのうち使わないようにする
use my_init::wait_until_enter;
use my_init::CONFIG;

use ljx_high_speed_communication::converter_raw_to_image;
use ljx_high_speed_communication::ProfileWriter;

use converter_raw_to_image::{convert_ljx_data_to_images, LjxDataConverterConfig};

fn main() -> anyhow::Result<()> {
    my_init::init();
    info!("logger initialized");

    let (instrumentation_config, converter_config) = get_configs()?;
    instrumentation_plz(instrumentation_config);
    info!("instrumentation_completed");

    convert_ljx_data_to_images(converter_config)?;
    Ok(())
}

fn get_configs() -> anyhow::Result<(InstrumentationConfig, LjxDataConverterConfig)> {
    dotenv().ok();
    let mut instrumentation_config =
        match envy::prefixed("InstrumentationConfig_").from_env::<InstrumentationConfig>() {
            Ok(config) => config,

            Err(error) => panic!("{:#?}", error),
        };

    let date = my_init::get_time_string();
    let save_path = CONFIG.save_dir.clone() + "/raw_profile" + &date + ".hex";
    instrumentation_config.set_save_path(save_path.clone());

    let mut converter_config =
        match envy::prefixed("LjxDataConverterConfig").from_env::<LjxDataConverterConfig>() {
            Ok(config) => config,

            Err(error) => panic!("{:#?}", error),
        };
    converter_config.set_ljx_data_path(save_path);

    Ok((instrumentation_config, converter_config))
}

// 検査コードも後々分離する
#[derive(Deserialize, Debug)]
struct InstrumentationConfig {
    save_dir: String,
    save_path: Option<String>,
    ljx_profile_data_num: usize,
    ljx_fetch_brightness_data: bool,
}

impl InstrumentationConfig {
    fn set_save_path(&mut self, path: String) {
        self.save_path = Some(path);
    }
}

fn instrumentation_plz(config: InstrumentationConfig) {
    let (mut interface, rx) = match LjxIf::create() {
        Ok(t) => t,
        Err(err) => panic!(
            "Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",
            err
        ),
    };

    info!("LJXインターフェースの作成");
    // rxからの受信データをパース⇒保存するスレッドを建てる
    // let profile_writer = ProfileWriter::new(rx, "./output".to_string(), 3200, true);
    let _profile_writer = ProfileWriter::new(
        rx,
        config.save_dir.clone(),
        config.ljx_profile_data_num,
        config.ljx_fetch_brightness_data,
    );
    info!("Profile_Writerの作成");

    wait_until_enter();

    match interface.open_ethernet(CONFIG.ljx_ip_address, CONFIG.ljx_port) {
        Ok(_t) => {}
        Err(err) => panic!("{:?}", err),
    }

    wait_until_enter();
    // ここでプロファイルデータを取得
    match interface.initialize_communication(CONFIG.ljx_high_speed_port) {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    wait_until_enter();

    match interface.pre_start_communication() {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    // wait_until_enter();

    match interface.start_communication() {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    // wait_until_enter();
    sleep(Duration::from_millis(5000));

    match interface.stop_communication() {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}
