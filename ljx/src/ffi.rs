#![allow(dead_code)] 
#![allow(non_snake_case)]

// pub type CHAR = ::std::os::raw::c_char;
pub type BYTE = ::std::os::raw::c_uchar;
// pub type SHORT = ::std::os::raw::c_short;
pub type WORD = ::std::os::raw::c_ushort;
pub type LONG = ::std::os::raw::c_int;
pub type DWORD = ::std::os::raw::c_uint;
// pub type FLOAT = ::std::os::raw::c_float;
// pub type DOUBLE = ::std::os::raw::c_double;

use std::sync::mpsc;
use crate::ReceiveData;

#[link(name = "bridge", kind = "static")]
#[allow(improper_ctypes)]
extern "C" {
    pub(crate) fn make_bridge_callback(target: *mut mpsc::Sender<ReceiveData>, cb: RustCallback)-> HighSpeedDataCommunicationCallback;
}
#[link(name = "LJX8_IF", kind = "static")]
extern "C" {
    pub(crate) fn LJX8IF_Initialize()-> LONG;
    pub(crate) fn LJX8IF_Finalize()-> LONG;
    pub(crate) fn LJX8IF_GetVersion() -> LJX8IF_VERSION_INFO;
    
    // Ethernet通信開始
    // LONG LJX8IF_EthernetOpen(LONG lDeviceId, LJX8IF_ETHERNET_CONFIG* pEthernetConfig);
    pub(crate) fn LJX8IF_EthernetOpen(
        lDeviceId:LONG,
        pEthernetConfig: *mut LJX8IF_ETHERNET_CONFIG
    )->LONG;
    // Ethernet通信切断
    // LONG LJX8IF_CommunicationClose(LONG lDeviceId);
    pub(crate) fn LJX8IF_CommunicationClose(
        lDeviceId:LONG,
    )->LONG;

    // 高速データ通信初期化
    // LONG LJX8IF_InitializeHighSpeedDataCommunication(
    //     LONG lDeviceId,
    //     LJX8IF_ETHERNET_CONFIG* pEthernetConfig, 高速データ通信用のコンフィグ
    //     WORD wHighSpeedPortNo,
    //     void (*pCallBack)(BYTE*, DWORD, DWORD,DWORD, DWORD),
    //     DWORD dwProfileCount,
    //     DWORD dwThreadId);
    pub(crate) fn LJX8IF_InitializeHighSpeedDataCommunication(
        lDeviceId:LONG,
        pEthernetConfig: *mut LJX8IF_ETHERNET_CONFIG,
        wHighSpeedPortNo: WORD,
        // callback関数 高速通信によってデータを受信した際に呼び出す
        pCallBack: HighSpeedDataCommunicationCallback,
        dwProfileCount:DWORD,
        dwThreadId:DWORD
    )->LONG;
    // // 高速データ通信開始準備要求
    // LONG LJX8IF_PreStartHighSpeedDataCommunication(
    //     LONG lDeviceId,
    //     LJX8IF_HIGH_SPEED_PRE_START_REQ* pReq,
    //     LJX8IF_PROFILE_INFO*pProfileInfo);
    pub(crate) fn LJX8IF_PreStartHighSpeedDataCommunication(
        lDeviceId:LONG,
        pReq: *mut LJX8IF_HIGH_SPEED_PRE_START_REQ,
        pProfileInfo: *mut LJX8IF_PROFILE_INFO,
    )->LONG;
    // 高速データ通信開始
    // LONG LJX8IF_StartHighSpeedDataCommunication(LONG lDeviceId);
    pub(crate) fn LJX8IF_StartHighSpeedDataCommunication(
        lDeviceId:LONG,        
    )->LONG;
    // 高速データ通信停止
    // LONG LJX8IF_StopHighSpeedDataCommunication(LONG lDeviceId);
    pub(crate) fn LJX8IF_StopHighSpeedDataCommunication(
        lDeviceId:LONG,
    )->LONG;
    // 高速データ通信終了
    // LONG LJX8IF_FinalizeHighSpeedDataCommunication(LONG lDeviceId);
    pub(crate) fn LJX8IF_FinalizeHighSpeedDataCommunication(
        lDeviceId:LONG,
    )->LONG;
    // 換算係数取得
    // LONG LJX8IF_GetZUnitSimpleArray(LONG lDeviceId, WORD* pwZUnit);
    pub(crate) fn LJX8IF_GetZUnitSimpleArray(
        lDeviceId:LONG,
        pwZUnit: *mut WORD,
    )->LONG;
}

pub fn check_return_code(return_code:LONG) -> anyhow::Result<()>{
    // 大きな型から小さな型へのキャスト。変換の際に上位ビットが切り捨てられる
    let return_code_u16 = return_code as u16;
    match return_code_u16 {
        0x0000 => return Ok(()),
        0x1000 => return Err(anyhow::anyhow!("Failed to open the communication path")),
        0x1001 => return Err(anyhow::anyhow!("The communication path was not established.")),
        0x1002 => return Err(anyhow::anyhow!("Failed to send the command.")),
        0x1003 => return Err(anyhow::anyhow!("Failed to receive a response.")),
        0x1004 => return Err(anyhow::anyhow!("A timeout occurred while waiting for the response.")),
        0x1005 => return Err(anyhow::anyhow!("Failed to allocate memory.")),
        0x1006 => return Err(anyhow::anyhow!("An invalid parameter was passed.")),
        0x1007 => return Err(anyhow::anyhow!("The received response data was invalid")),
        
        0x1009 => return Err(anyhow::anyhow!("High-speed communication initialization could not be performed.")),
        0x100A => return Err(anyhow::anyhow!("High-speed communication was initialized.")),
        0x100B => return Err(anyhow::anyhow!("Error already occurred during high-speed communication (for high-speed communication)")),
        0x100C => return Err(anyhow::anyhow!("The buffer size passed as an argument is insufficient. ")),

        0x8081 => return Err(anyhow::anyhow!("送信開始位置と指定されたデータが存在しない")),
        0x80A1 => return Err(anyhow::anyhow!("既に高速データ通信を行っている")),
        
        0x80A0 => return Err(anyhow::anyhow!("高速データ通信用の接続が確立していない")),
        0x80A2 => return Err(anyhow::anyhow!("高速データ通信開始前準備が行われていない")),
        0x80A3 => return Err(anyhow::anyhow!("高速データ通信開始前準備が行われていない")),
        0x80A4 => return Err(anyhow::anyhow!("送信対象のデータがクリアされた")),
        _ => return Err(anyhow::anyhow!("想定外のリターンコード")),
    }
}

// void (*pCallBack) (
// BYTE* pBuffer, DWORD dwSize, DWORD dwCount, DWORD dwNotify,
// DWORD dwUser);
pub(crate) type HighSpeedDataCommunicationCallback = extern "C" fn(
    pBuffer: *mut BYTE,
    dwSize: DWORD,
    dwCount: DWORD,
    dwNotify: DWORD,
    dwUser:DWORD,
);

pub(crate) type RustCallback = extern "C" fn(
    target:*mut mpsc::Sender<ReceiveData>,
    pBuffer: *mut BYTE,
    dwSize: DWORD,
    dwCount: DWORD,
    dwNotify: DWORD,
    dwUser:DWORD,
);


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LJX8IF_VERSION_INFO {
    pub nMajorNumber: LONG,
    pub nMinorNumber: LONG,
    pub nRevisionNumber: LONG,
    pub nBuildNumber: LONG,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LJX8IF_ETHERNET_CONFIG {
    pub abyIpAddress: [BYTE; 4usize],
    pub wPortNo: WORD,
    pub reserve: [BYTE; 2usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LJX8IF_HIGH_SPEED_PRE_START_REQ {
    pub bySendPosition: BYTE,
    pub reserve: [BYTE; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LJX8IF_PROFILE_INFO {
    pub byProfileCount: BYTE,
    pub reserve1: BYTE,
    pub byLuminanceOutput: BYTE,
    pub reserve2: BYTE,
    pub wProfileDataCount: WORD,
    pub reserve3: [BYTE; 2usize],
    pub lXStart: LONG,
    pub lXPitch: LONG,
}

impl LJX8IF_PROFILE_INFO {
    pub fn new() -> Self  {
        Self{
            byProfileCount: 0,
            reserve1: 0,
            byLuminanceOutput: 0,
            reserve2: 0,
            wProfileDataCount: 0,
            reserve3: [0,0],
            lXStart: 0,
            lXPitch: 0,
        }
    }
}