#![allow(dead_code)]
#![allow(non_snake_case)]
use std::sync::mpsc;

mod ffi;
mod types;

pub use types::*;

#[no_mangle]
extern "C" fn send_data(
    target: *mut mpsc::Sender<ReceiveData>,
    pBuffer: *mut ffi::BYTE,
    // ヘッダ-データ-フッタを1単位として、いくつ入っているか
    dwSize: ffi::DWORD,
    // pBuffer内のプロファイル数
    dwCount: ffi::DWORD,
    // 高速データ通信の中断やバッチ測定の区切りを通知します
    dwNotify: ffi::DWORD,
    // スレッドID = dwThreadId
    dwUser: ffi::DWORD,
) {
    // TODO:時間計測用のロガー設置
    // println!("I'm called from C with value");
    let receive_slice = unsafe { std::slice::from_raw_parts(pBuffer, (dwSize*dwCount).try_into().unwrap()) }; 
    let vec = receive_slice.to_vec();
    let data  = ReceiveData{
        data:vec,
        count:dwCount,
        notify:dwNotify,
        user:dwUser
    };
    unsafe {
        (*target).send(data).unwrap();
    }
}


pub struct LjxIf {
    sender:Box<mpsc::Sender<ReceiveData>>,
    callback_fn:ffi::HighSpeedDataCommunicationCallback,

    is_initialized: bool,
    is_eathernet_open:bool,
    is_initialized_communication:bool,
    is_pre_start_communication:bool,
    is_communicating:bool,
    // 定数
    device_id:i32,//現状デバイスを複数つなぐことを考えていないので0
    profile_batch_size:u32,
    thread_id:u32,

    ip_config:Option<ffi::LJX8IF_ETHERNET_CONFIG>,
    high_speed_port:Option<u16>,

}

impl LjxIf {
    pub fn create() ->anyhow::Result<(Self,mpsc::Receiver<ReceiveData>)> {
        unsafe{ 
            let receive_code = ffi::LJX8IF_Initialize();
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_Initialize:{:?}",err)),
            }
        };
        let (tx,rx) = mpsc::channel();
        let mut sender = Box::new(tx);
        let cb = unsafe{ ffi::make_bridge_callback(&mut *sender, send_data)};

        Ok((Self {
            sender:sender,
            callback_fn:cb,

            is_initialized: true,
            is_eathernet_open:false,
            is_initialized_communication:false,
            is_pre_start_communication:false,
            is_communicating:false,
            device_id:0,
            profile_batch_size:20,
            thread_id:0,
            ip_config:None,
            high_speed_port:None,
        }, rx))
    }

    pub fn get_dll_info(&self) -> anyhow::Result<DllInfo>{
        if !self.is_initialized {
            return Err(anyhow::anyhow!("Error LjxIf::when get_dll_info: Dll not initialized"))
        };
        let info = unsafe {ffi::LJX8IF_GetVersion()};

        Ok(DllInfo {
            major_number: info.nMajorNumber,
            minor_number: info.nMinorNumber,
            revision_number: info.nRevisionNumber,
            build_number: info.nBuildNumber,
        })

    }

    pub fn open_eathernet(&mut self, ip_address: [u8;4],port:u16) -> anyhow::Result<()>{
        let mut ip_config = ffi::LJX8IF_ETHERNET_CONFIG{
            abyIpAddress: ip_address,
            wPortNo: port,
            reserve: [0,0],
        };
        unsafe{ 
            let receive_code = ffi::LJX8IF_EthernetOpen(self.device_id,&mut ip_config); 
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_EthernetOpen:{:?}",err)),
            }
        };

        self.is_eathernet_open = true;
        self.ip_config = Some(ip_config);
        Ok(())
    }

    pub fn initialize_communication(&mut self,high_speed_port:u16) -> anyhow::Result<()>{
        if !self.is_eathernet_open {
            return Err(anyhow::anyhow!("Error not eathernet open when initialize_communication"))
        }
        unsafe {
            let receive_code = ffi::LJX8IF_InitializeHighSpeedDataCommunication(
                self.device_id,
                &mut self.ip_config.unwrap(),
                high_speed_port,
                self.callback_fn,
                self.profile_batch_size,
                self.thread_id
            ); 
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",err)),
            }
        }
        self.is_initialized_communication = true;
        Ok(())
    }


    pub fn pre_start_communication(&mut self) -> anyhow::Result<()> {
        if !self.is_initialized_communication {
            return Err(anyhow::anyhow!("Error not initialized communication when pre_start_communication"))
        }

        let mut profile_info = ffi::LJX8IF_PROFILE_INFO::new();
        let mut start_request = ffi::LJX8IF_HIGH_SPEED_PRE_START_REQ{
            bySendPosition: 0,
            // 0：前回送信完了位置から（初回であれば最古データから）、
            // 1：最古デー タから（取り直し）、
            // 2：次のデータから
            reserve: [0,0,0],
        };

        unsafe {
            let receive_code = ffi::LJX8IF_PreStartHighSpeedDataCommunication(
                self.device_id, &mut start_request,&mut profile_info);
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_PreStartHighSpeedDataCommunication:{:?}",err)),
            }

            println!("Success LJX8IF_PreStartHighSpeedDataCommunication");
            println!("profile_info: {:?}",profile_info);
        }
        
        self.is_pre_start_communication = true;
        Ok(())
    }

    pub fn start_communication(&mut self) -> anyhow::Result<()>{
        if !self.is_pre_start_communication {
            return Err(anyhow::anyhow!("Error not initialized communication when start_communication"))
        }
        if !self.is_pre_start_communication {
            return Err(anyhow::anyhow!("Error already communicating when start_communication"))
        }

        unsafe {
            let receive_code = ffi::LJX8IF_StartHighSpeedDataCommunication(self.device_id); 
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_StartHighSpeedDataCommunication:{:?}",err)),
            }
        }
        
        self.is_communicating = true;
        Ok(())
    }

    pub fn stop_communication(&mut self) -> anyhow::Result<()>{
        if !self.is_communicating {
            return Err(anyhow::anyhow!("Error not communicating when stop_communication"))
        }

        unsafe {
            let receive_code = ffi::LJX8IF_StopHighSpeedDataCommunication(self.device_id); 
            match ffi::check_return_code(receive_code) {
                Ok(_) => {},
                Err(err) => return Err(anyhow::anyhow!("Error when ffi::LJX8IF_StopHighSpeedDataCommunication:{:?}",err)),
            }
        }
        
        self.is_communicating = true;
        Ok(())
    }
    
}

impl Drop for LjxIf {
    fn drop(&mut self){
        println!("destruct from drop"); 

        // if self.is_pre_start_communication{
        //     // 特に必要な処理はない
        // }

        if self.is_initialized_communication {
            unsafe{ ffi::LJX8IF_FinalizeHighSpeedDataCommunication(self.device_id)};
        }

        if self.is_eathernet_open {
            unsafe{ ffi::LJX8IF_CommunicationClose(self.device_id)};
        };

        if self.is_initialized {
            unsafe { ffi::LJX8IF_Finalize() };
            println!("DLL finalized"); 
        };
    }
}

