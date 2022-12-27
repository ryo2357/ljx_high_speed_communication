use std::fs::File;
use std::io::{self, Read, Write, BufReader,BufWriter};

mod lib;
use lib::dummy::make_dummy;

fn main(){
    let num_profile = 10;
    let num_data = 3200;
    let vec= make_dummy(num_profile,num_data);

    // 所有権確認用
    println!("ダミーデータ生成：{:?}",vec.len() );

    wait_until_enter();

    let path = "output/dummy.hex";

    let mut data_writer = DataWriter::create(path,num_data).unwrap();

    for profile in vec.chunks(4*(num_data+7)) {
        data_writer.push_profile(profile);
    }

    // 所有権確認用
    println!("ダミーデータ確認：{:?}",vec.len() );
    
}

struct DataWriter {
    writer:BufWriter<File>,
    data_end:usize
}

impl DataWriter {
    fn create(path:&str,data_num:usize) -> anyhow::Result<Self>{
        let mut file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        let data_header = data_num.to_le_bytes();
        writer.write(&data_header);
        let data_end = 24+4*data_num;

        Ok(Self{
            writer: writer,
            data_end: data_end,
        })
    }

    fn push_profile(&mut self,profile:&[u8] ){
        let header = &profile[0..16];
        self.writer.write(header);
        let data = &profile[24..self.data_end];
        self.writer.write(data);
        println!("ヘッダ[{}]１バイト目：{:?}",header.len(),header[0]);
        println!("データ[{}]１バイト目：{:?}",data.len(),data[0]);
        println!("データ[{}]最終バイト：{:?}",data.len(),data.last().unwrap());
    }
}

fn wait_until_enter(){
    print!("press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    // print!("{}", input);
}