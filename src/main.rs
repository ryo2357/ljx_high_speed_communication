
use std::thread;
use std::sync::mpsc;
use std::io::Write;

use ljx::{LjxIf,ReceiveData};
use log::{info};

fn wait_until_enter(){
    print!("wait until press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn data_receive_loop(rx: mpsc::Receiver<ReceiveData>){
    loop {
        let vec = rx.recv().unwrap();

    }
}

fn main() {
    mylogger::init();
    info!("logger initialized");

    let (mut interface, mut rx) = match LjxIf::create(){
        Ok(t) => t,
        Err(err) => panic!("Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",err),
    };

    // rxからの受信データをパース⇒保存するスレッドを建てる
    thread::spawn(move||data_receive_loop(rx));
    
    wait_until_enter();

    match interface.open_ethernet([192,168,0,1], 3000) {
        Ok(t) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();
    // ここでプロファイルデータを取得
    match interface.initialize_communication(4000){
        Ok(t) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();

    match interface.pre_start_communication(){
        Ok(t) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();

    match interface.start_communication(){
        Ok(t) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();

    match interface.stop_communication(){
        Ok(t) => {},
        Err(err) => panic!("{:?}",err),

    }


}