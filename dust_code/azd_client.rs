// AzdUdpThreadに機能を持たせすぎで分かりにくくなってる

use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

pub struct AzdBlockingClient {
    // inner: AZD_Client,
    rt: Runtime,
}

impl AzdBlockingClient {
    pub fn connect() -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let (inner, sender, receiver) = rt.block_on(AzdUdpClient::connect(addr))?;

        Ok(Self { rt })
    }

    // pub fn revert_to_standard_position() -> anyhow::Result<()> {}
}

type ReceiveBuff = [u8; 56];
type SendBuff = [u8; 40];

struct AzdUdpThread {
    sender_handle: JoinHandle<anyhow::Result<()>>,
    receive_handle: JoinHandle<anyhow::Result<()>>,
}
impl AzdUdpThread {
    async fn connect(
        addr: &str,
    ) -> anyhow::Result<(Self, Sender<SendBuff>, Receiver<ReceiveBuff>)> {
        let (send_tx, mut send_rx) = mpsc::channel::<SendBuff>();
        let (recv_tx, recv_rx) = mpsc::channel::<ReceiveBuff>();

        // addr: "127.0.0.1:34254"
        let socket = UdpSocket::bind(addr).await?;
        // socket.set_nonblocking(true)?;
        let socket_receiver = Arc::new(socket);
        let socket_sender = socket_receiver.clone();

        let sender_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Some(buff) = send_rx.recv().await {
                let len = socket_sender.send(&buff).await?;
                println!("{:?} bytes sent to UDP", len);
            }
            Ok(())
        });
        #[allow(unreachable_code)]
        let receive_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let mut buf: ReceiveBuff = [0; 56];
            loop {
                let len = socket_receiver.recv(&mut buf).await?;
                // let len = socket_sender.send(&array).await?;
                println!("{:?} bytes received from UDP", len);
                recv_tx.send(buf).await?;
            }
            Ok(())
        });
        Ok((
            Self {
                sender_handle,
                receive_handle,
            },
            send_tx,
            recv_rx,
        ))
    }
}
