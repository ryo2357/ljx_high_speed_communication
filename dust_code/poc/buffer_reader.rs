use std::fs::File;
use std::io::{self, Read, Write, BufReader,BufWriter};


fn main(){

    let path = "output/dummy.hex";

    let mut data_reader = DataReader::create(path).unwrap();
    
    data_reader.check_data().unwrap();


}

struct DataReader {
    reader:BufReader<File>,
    profile_size:usize
}

impl DataReader {
    fn create(path:&str) -> anyhow::Result<Self>{
        let mut file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        let mut header = [0;8];
        reader.read(&mut header);

        let data_num = usize::from_le_bytes(header);
        println!("data_num:{}",data_num);

        Ok(Self{
            reader: reader,
            profile_size: 16+ data_num*4,
        })
    }

    fn check_data(&mut self) -> anyhow::Result<()> {
        let mut profile:Vec<u8> = Vec::new();
        profile.resize(self.profile_size,0);
        loop {
            match self.reader.read_exact(&mut profile) {
                Ok(()) => {
                    // println!("プロファイル数：{:?}",profile.len());
                    println!("ヘッダ１バイト目{:?}",profile[0]);
                    println!("データ１バイト目{:?}",profile[16]);
                    println!("データ最終１バイト目{:?}",profile.last().unwrap());
                },
                // buf: &mut [u8] をフィルできなかった場合、ErrorKind::UnexpectedEof
                Err(_)=> break,
            }
        }

        Ok(())
    }

    // fn push_profile(&mut self,profile:&[u8] ){
    //     let header = &profile[0..16];
    //     self.writer.write(header);
    //     let data = &profile[24..self.data_end];
    //     self.writer.write(data);
    //     println!("ヘッダ[{}]、１バイト目:{:?}",header.len(),header[0]);
    //     println!("データ[{}]、１バイト目:{:?}",data.len(),data[0]);
    // }
}

fn wait_until_enter(){
    print!("press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    // print!("{}", input);
}
