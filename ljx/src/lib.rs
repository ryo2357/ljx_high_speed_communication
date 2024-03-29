#![allow(dead_code)]
#![allow(non_snake_case)]
use log::info;
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
    let receive_slice =
        unsafe { std::slice::from_raw_parts(pBuffer, (dwSize * dwCount).try_into().unwrap()) };
    let vec = receive_slice.to_vec();
    let data = ReceiveData {
        data: vec,
        count: dwCount,
        notify: dwNotify,
        user: dwUser,
    };
    info!(
        "send data from ljx dwCount:{},dwNotify:{}",
        dwCount, dwNotify
    );
    unsafe {
        (*target).send(data).unwrap();
    }
}

pub struct LjxIf {
    // create()で設定
    is_initialized: bool,
    sender: Box<mpsc::Sender<ReceiveData>>,
    callback_fn: ffi::HighSpeedDataCommunicationCallback,

    // open_ethernet()で操作
    is_ethernet_open: bool,
    ip_config: Option<ffi::LJX8IF_ETHERNET_CONFIG>,

    // initialize_communication()で設定
    is_initialized_communication: bool,
    is_pre_start_communication: bool,
    is_communicating: bool,

    // 定数、create()で設定
    device_id: i32, //現状デバイスを複数つなぐことを考えていないので0
    profile_batch_size: u32,
    thread_id: u32,
    high_speed_port: Option<u16>,
}

impl LjxIf {
    pub fn create() -> anyhow::Result<(Self, mpsc::Receiver<ReceiveData>)> {
        unsafe {
            let receive_code = ffi::LJX8IF_Initialize();
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_Initialize:{:?}",
                        err
                    ))
                }
            }
        };
        let (tx, rx) = mpsc::channel();
        let mut sender = Box::new(tx);
        let cb = unsafe { ffi::make_bridge_callback(&mut *sender, send_data) };

        info!("create LjxIF");
        Ok((
            Self {
                is_initialized: true,
                sender: sender,
                callback_fn: cb,
                is_ethernet_open: false,
                is_initialized_communication: false,
                is_pre_start_communication: false,
                is_communicating: false,
                device_id: 0,
                profile_batch_size: 8000,
                thread_id: 0,
                ip_config: None,
                high_speed_port: None,
            },
            rx,
        ))
    }

    pub fn get_dll_info(&self) -> anyhow::Result<DllInfo> {
        if !self.is_initialized {
            return Err(anyhow::anyhow!(
                "Error LjxIf::when get_dll_info: Dll not initialized"
            ));
        };
        let info = unsafe { ffi::LJX8IF_GetVersion() };

        Ok(DllInfo {
            major_number: info.nMajorNumber,
            minor_number: info.nMinorNumber,
            revision_number: info.nRevisionNumber,
            build_number: info.nBuildNumber,
        })
    }

    pub fn open_ethernet(&mut self, ip_address: [u8; 4], port: u16) -> anyhow::Result<()> {
        let mut ip_config = ffi::LJX8IF_ETHERNET_CONFIG {
            abyIpAddress: ip_address,
            wPortNo: port,
            reserve: [0, 0],
        };
        unsafe {
            let receive_code = ffi::LJX8IF_EthernetOpen(self.device_id, &mut ip_config);
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_EthernetOpen:{:?}",
                        err
                    ))
                }
            }
        };
        info!("open eathenet");
        self.is_ethernet_open = true;
        self.ip_config = Some(ip_config);
        Ok(())
    }

    pub fn initialize_communication(&mut self, high_speed_port: u16) -> anyhow::Result<()> {
        if !self.is_ethernet_open {
            return Err(anyhow::anyhow!(
                "Error not ethernet open when initialize_communication"
            ));
        }
        unsafe {
            let receive_code = ffi::LJX8IF_InitializeHighSpeedDataCommunication(
                self.device_id,
                &mut self.ip_config.unwrap(),
                high_speed_port,
                self.callback_fn,
                self.profile_batch_size,
                self.thread_id,
            );
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_InitializeHighSpeedDataCommunication:{:?}",
                        err
                    ))
                }
            }
            // TODO:receive_codeの確認
            info!(
                "initialize high speed communication. receive_code: {:?}",
                receive_code
            );
        }
        self.is_initialized_communication = true;
        Ok(())
    }

    pub fn pre_start_communication(&mut self) -> anyhow::Result<()> {
        if !self.is_initialized_communication {
            return Err(anyhow::anyhow!(
                "Error not initialized communication when pre_start_communication"
            ));
        }

        let mut profile_info = ffi::LJX8IF_PROFILE_INFO::new();
        let mut start_request = ffi::LJX8IF_HIGH_SPEED_PRE_START_REQ {
            bySendPosition: 2,
            // 0：前回送信完了位置から（初回であれば最古データから）、
            // 1：最古データから（取り直し）、
            // 2：次のデータから ⇒ コントローラーに溜まっているデータは破棄される？
            reserve: [0, 0, 0],
        };

        unsafe {
            let receive_code = ffi::LJX8IF_PreStartHighSpeedDataCommunication(
                self.device_id,
                &mut start_request,
                &mut profile_info,
            );
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_PreStartHighSpeedDataCommunication:{:?}",
                        err
                    ))
                }
            }

            info!("profile_info: {:?}", profile_info);
            // TODO:どのようなプロファイルが返ってくるかテスト
            // 入力値とコントローラー内に保存されているデータの関係
        }
        info!("pre started high speed communication");

        self.is_pre_start_communication = true;
        Ok(())
    }

    pub fn start_communication(&mut self) -> anyhow::Result<()> {
        if !self.is_pre_start_communication {
            return Err(anyhow::anyhow!(
                "Error not initialized communication when start_communication"
            ));
        }
        if !self.is_pre_start_communication {
            return Err(anyhow::anyhow!(
                "Error already communicating when start_communication"
            ));
        }

        unsafe {
            let receive_code = ffi::LJX8IF_StartHighSpeedDataCommunication(self.device_id);
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_StartHighSpeedDataCommunication:{:?}",
                        err
                    ))
                }
            }
        }

        self.is_communicating = true;
        info!("start high speed communication");
        Ok(())
    }

    pub fn stop_communication(&mut self) -> anyhow::Result<()> {
        if !self.is_communicating {
            return Err(anyhow::anyhow!(
                "Error not communicating when stop_communication"
            ));
        }

        unsafe {
            let receive_code = ffi::LJX8IF_StopHighSpeedDataCommunication(self.device_id);
            match ffi::check_return_code(receive_code) {
                Ok(_) => {}
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Error when ffi::LJX8IF_StopHighSpeedDataCommunication:{:?}",
                        err
                    ))
                }
            }
        }

        info!("stop high speed communication");
        self.is_communicating = false;
        Ok(())
    }
}

impl Drop for LjxIf {
    fn drop(&mut self) {
        // info!("LjxFf destruct from drop");
        info!("LjxFfのDropよる終了処理");

        // if self.is_pre_start_communication{
        //     // 特に必要な処理はない
        // }
        if self.is_communicating {
            self.stop_communication().unwrap();
        }

        if self.is_initialized_communication {
            unsafe { ffi::LJX8IF_FinalizeHighSpeedDataCommunication(self.device_id) };
            info!("Communication finalized");
        }

        if self.is_ethernet_open {
            unsafe { ffi::LJX8IF_CommunicationClose(self.device_id) };
            info!("Ethernet finalized");
        };

        if self.is_initialized {
            unsafe { ffi::LJX8IF_Finalize() };
            info!("DLL finalized");
        };
    }
}
