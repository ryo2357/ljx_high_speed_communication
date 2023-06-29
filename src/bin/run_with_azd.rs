use std::thread::sleep;
use std::time::Duration;

use ljx::LjxIf;
use log::info;

use my_init::wait_until_enter;
use my_init::CONFIG;

extern crate ljx_high_speed_communication;

pub use ljx_high_speed_communication::ProfileWriter;

fn main() {
    my_init::init();
    info!("logger initialized");

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
        CONFIG.save_dir.clone(),
        CONFIG.ljx_profile_data_num,
        CONFIG.ljx_fetch_brightness_data,
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
