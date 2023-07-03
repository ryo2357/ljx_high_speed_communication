//  高速データ通信、バッチ測定
use std::thread;

use std::sync::mpsc;

use std::fs::File;
use std::io::{BufWriter, Write};

use ljx::ReceiveData;
use log::info;
use std::thread::JoinHandle;

struct DataWriter {
    writer: BufWriter<File>,
    data_num: usize,
    data_end: usize,
}

impl DataWriter {
    fn create(path: String, data_num: usize, brightness: bool) -> anyhow::Result<Self> {
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);

        let data_end = match brightness {
            false => 24 + 4 * data_num,
            true => 24 + 8 * data_num,
        };

        Ok(Self {
            writer,
            data_num,
            data_end,
        })
    }

    fn push_profile(&mut self, profile: &[u8]) {
        let header = &profile[0..16];
        self.writer.write_all(header).unwrap();
        let data = &profile[24..self.data_end];
        self.writer.write_all(data).unwrap();
    }

    fn push_data(&mut self, data: Vec<u8>) {
        for profile in data.chunks(4 * (self.data_num + 7)) {
            self.push_profile(profile);
        }
    }
}

#[allow(dead_code)]
pub struct ProfileWriter {
    inner: JoinHandle<anyhow::Result<()>>,
}

impl ProfileWriter {
    pub fn new(
        rx: mpsc::Receiver<ReceiveData>,
        save_path: String,
        data_num: usize,
        fetch_brightness_data: bool,
    ) -> anyhow::Result<Self> {
        // let date = my_init::get_time_string();
        // let path = save_dir + "/raw_profile" + &date + ".hex";
        // let fetch_brightness_data = false;
        // let mut data_writer = DataWriter::create(path, 3200, fetch_brightness_data).unwrap();
        let mut data_writer =
            DataWriter::create(save_path, data_num, fetch_brightness_data).unwrap();

        #[allow(unreachable_code)]
        let thread: JoinHandle<anyhow::Result<()>> = thread::spawn(move || {
            loop {
                let receive_data = rx.recv().unwrap();
                // TODO:dwNotifyの値によってフローを変更する必要があるか？
                data_writer.push_data(receive_data.data);
                info!("received {} data written", receive_data.count);
            }

            Ok(())
        });

        Ok(Self { inner: thread })
    }
}
