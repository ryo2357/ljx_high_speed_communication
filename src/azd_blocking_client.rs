use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;

// 送受信するデータ形状の宣言
type ReceiveBuff = [u8; 56];
type SendBuff = [u8; 40];

struct ReceiveData {
    remote_io: u16,
    driving_data: u16,
    fixed_io: u16,
    current_alarm: u16,
    detection_position: u32,
    detection_speed: u32,
    command_position: u32,
    torque_monitor: u16,
    cst_driving_current: u16,
    information: u32,
    reservation: u16,
    read_parameter_id: u16,
    rw_status: u16,
    write_parameter_id: u16,
    read_data: u32,
    optional_monitor_0: u32,
    optional_monitor_1: u32,
    optional_monitor_2: u32,
    optional_monitor_3: u32,
}
impl ReceiveData {
    fn convert(buff: ReceiveBuff) -> Self {
        let remote_io = u16::from_le_bytes([buff[0], buff[1]]);
        let driving_data = u16::from_le_bytes([buff[2], buff[3]]);
        let fixed_io = u16::from_le_bytes([buff[4], buff[5]]);
        let current_alarm = u16::from_le_bytes([buff[6], buff[7]]);
        let detection_position = u32::from_le_bytes([buff[8], buff[9], buff[10], buff[11]]);
        let detection_speed = u32::from_le_bytes([buff[12], buff[13], buff[14], buff[15]]);
        let command_position = u32::from_le_bytes([buff[16], buff[17], buff[18], buff[19]]);
        let torque_monitor = u16::from_le_bytes([buff[20], buff[21]]);
        let cst_driving_current = u16::from_le_bytes([buff[22], buff[23]]);
        let information = u32::from_le_bytes([buff[24], buff[25], buff[26], buff[27]]);
        let reservation = u16::from_le_bytes([buff[28], buff[29]]);
        let read_parameter_id = u16::from_le_bytes([buff[30], buff[31]]);
        let rw_status = u16::from_le_bytes([buff[32], buff[33]]);
        let write_parameter_id = u16::from_le_bytes([buff[34], buff[35]]);
        let read_data = u32::from_le_bytes([buff[36], buff[37], buff[38], buff[39]]);
        let optional_monitor_0 = u32::from_le_bytes([buff[40], buff[41], buff[42], buff[43]]);
        let optional_monitor_1 = u32::from_le_bytes([buff[44], buff[45], buff[46], buff[47]]);
        let optional_monitor_2 = u32::from_le_bytes([buff[48], buff[49], buff[50], buff[51]]);
        let optional_monitor_3 = u32::from_le_bytes([buff[52], buff[53], buff[54], buff[55]]);
        Self {
            remote_io,
            driving_data,
            fixed_io,
            current_alarm,
            detection_position,
            detection_speed,
            command_position,
            torque_monitor,
            cst_driving_current,
            information,
            reservation,
            read_parameter_id,
            rw_status,
            write_parameter_id,
            read_data,
            optional_monitor_0,
            optional_monitor_1,
            optional_monitor_2,
            optional_monitor_3,
        }
    }
}

// 同期コードから呼び出す構造体
// tokioランタイムと非同期コードのメソッドをラップして同期コードにしたメソッドを実装
pub struct AzdBlockingClient {
    // inner: AZD_Client,
    rt: Runtime,
}

impl AzdBlockingClient {
    pub fn connect(addr: &str) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // addr="127.0.0.1:34254"
        let (inner, sender) = rt.block_on(AzdUdpSocket::connect(addr))?;

        Ok(Self { rt })
    }

    // pub fn revert_to_standard_position() -> anyhow::Result<()> {}
}

// tokioで実装したAzdのクライアント
pub struct AzdAsyncClient {
    socket: AzdUdpSocket,
    state: Arc<Mutex<AzdState>>,
}
impl AzdAsyncClient {
    async fn connect(addr: &str) -> anyhow::Result<Self> {
        let (socket, mut receiver) = AzdUdpSocket::connect(addr).await?;
        let state: Arc<Mutex<AzdState>> = Default::default();

        #[allow(unreachable_code)]
        let receive_state = state.clone();
        let _receive_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Some(buff) = receiver.recv().await {
                let mut state_inner = receive_state.lock().unwrap();
                state_inner.update(buff);
            }
            Ok(())
        });

        Ok(Self { socket, state })
    }
}

// Azdの状態管理構造体
struct AzdState {
    last_receive_buff: ReceiveBuff,
    foo: i32,
    bar: f32,
}
impl Default for AzdState {
    fn default() -> Self {
        Self {
            last_receive_buff: [0; 56],
            foo: 0,
            bar: 0.0f32,
        }
    }
}
impl AzdState {
    fn update(&mut self, buff: ReceiveBuff) -> anyhow::Result<()> {
        Ok(())
    }
    fn make_revert_command(&self) -> anyhow::Result<SendBuff> {
        Ok([0; 40])
    }
    fn make_start_command(&self) -> anyhow::Result<SendBuff> {
        Ok([0; 40])
    }
}

// ソケット通信と送受信のスレッドを生成する構造体
struct AzdUdpSocket {
    inner: Arc<UdpSocket>,
}
impl AzdUdpSocket {
    async fn connect(addr: &str) -> anyhow::Result<(Self, Receiver<ReceiveBuff>)> {
        let socket = UdpSocket::bind(addr).await?;
        let socket_receiver = Arc::new(socket);
        let socket_sender = socket_receiver.clone();
        let (tx, rx) = mpsc::channel::<ReceiveBuff>(5);

        #[allow(unreachable_code)]
        let _receive_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let mut buf: ReceiveBuff = [0; 56];
            loop {
                let len = socket_receiver.recv(&mut buf).await?;
                // let len = socket_sender.send(&array).await?;
                println!("{:?} bytes received from UDP", len);
                tx.send(buf).await?;
            }
            Ok(())
        });

        Ok((
            Self {
                inner: socket_sender,
            },
            rx,
        ))
    }

    async fn send_message(&self, buff: SendBuff) -> anyhow::Result<()> {
        let len = self.inner.send(&buff).await?;
        println!("{:?} bytes sent to UDP", len);
        Ok(())
    }
}
