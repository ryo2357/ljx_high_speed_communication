//  高速データ通信、バッチ測定
use std::thread;
use std::sync::mpsc;

use std::time::Duration;
use std::thread::sleep;

use std::fs::File;
use std::io::{Write,BufWriter};

use ljx::{LjxIf,ReceiveData};
use log::{info};

fn wait_until_enter(){
    print!("wait until press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

struct DataWriter {
    writer:BufWriter<File>,
    data_num:usize,
    data_end:usize
}

impl DataWriter {
    fn create(path:String,data_num:usize) -> anyhow::Result<Self>{
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        
        let data_end = 24+4*data_num;

        Ok(Self{
            writer: writer,
            data_num: data_num,
            data_end: data_end,
        })
    }

    fn push_profile(&mut self,profile:&[u8] ){
        let header = &profile[0..16];
        self.writer.write(header).unwrap();
        let data = &profile[24..self.data_end];
        self.writer.write(data).unwrap();
    }

    fn push_data(&mut self,data:Vec<u8>){
        for profile in data.chunks(4*(self.data_num+7)) {
            self.push_profile(profile);
        }
    }
}

fn data_receive_loop(rx: mpsc::Receiver<ReceiveData>){

    let date = mylogger::get_time_string();
    let path = "output/data_".to_string() + &date + ".hex";
    let mut data_writer = DataWriter::create(path,3200).unwrap();


    loop {
        let receive_data = rx.recv().unwrap();
        // TODO:dwNotifyの値によってフローを変更する必要があるか？
        data_writer.push_data(receive_data.data);
        info!("received {} data written",receive_data.count);
    }
}

fn main() {
    mylogger::init();
    info!("logger initialized");
    
    info!("Rustでの通信検証");

    let (mut interface, rx) = match LjxIf::create(){
        Ok(t) => t,
        Err(err) => panic!("Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",err),
    };

    // rxからの受信データをパース⇒保存するスレッドを建てる
    thread::spawn(move||data_receive_loop(rx));
    
    wait_until_enter();

    match interface.open_ethernet([192,168,0,1], 24691) {
        Ok(_t) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();
    // ここでプロファイルデータを取得
    match interface.initialize_communication(24692){
        Ok(_) => {},
        Err(err) => panic!("{:?}",err),
    }

    wait_until_enter();

    match interface.pre_start_communication(){
        Ok(_) => {},
        Err(err) => panic!("{:?}",err),
    }

    // wait_until_enter();

    match interface.start_communication(){
        Ok(_) => {},
        Err(err) => panic!("{:?}",err),
    }

    // wait_until_enter();
    sleep(Duration::from_millis(5000));


    match interface.stop_communication(){
        Ok(_) => {},
        Err(err) => panic!("{:?}",err),

    }


}