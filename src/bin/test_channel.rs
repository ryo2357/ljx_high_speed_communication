use log::info;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::Duration;

#[derive(Debug, Default)]
struct State {
    num: usize,
    log: usize,
    wake: bool,
}

#[tokio::main]
async fn main() {
    // Establish a connection to the server
    my_init::init();
    let state: Arc<Mutex<State>> = Default::default();
    let handle_state = state.clone();

    let _handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000)).await;
            let mut state = handle_state.lock().unwrap();
            state.num += 1;
            state.wake = true;
        }
    });

    let interrupt_state = state.clone();

    let _interrupt_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(500)).await;
            let mut state = interrupt_state.lock().unwrap();
            if state.wake {
                state.wake = false;
                info!("wake");
                state.log += state.num;
            }
        }
    });

    let _info_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(2000)).await;
            info!("test {:?}", state.lock().unwrap());
        }
    });

    // loop {}
    std::thread::yield_now();
}
