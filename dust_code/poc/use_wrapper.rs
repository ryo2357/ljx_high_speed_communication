use ljx::LjxIf;

// #[macro_use]
// extern crate log;

use log::info;

fn main() {
    my_init::init();

    info!("test");

    let (mut interface, mut rx) = match LjxIf::create() {
        Ok(t) => t,
        Err(err) => panic!(
            "Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",
            err
        ),
    };
    let info = match interface.get_dll_info() {
        Ok(t) => t,
        Err(err) => panic!(
            "Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",
            err
        ),
    };

    println!("DLL Info: {:?}", info);
}
