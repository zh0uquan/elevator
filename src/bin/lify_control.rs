use anyhow::Result;
use elevator::services::controller::ControllerService;
use elevator::services::scheduler::SchedulerEventLayer;
use elevator::services::udp_event::UdpEventLayer;
use elevator::strategies::scan::{ScanStrategy, SchedulerState};
use elevator::transition::{ElevatorState, IntoBoxedTransition, PreStart};
use elevator::types::cmd::Command;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tower::{Service, ServiceBuilder, ServiceExt};

const UDP_MAX_SIZE: usize = 65535;
const CONTROL_ADDRESS: &str = "127.0.0.1:11000";
const LIFTY_ADDRESS: &str = "127.0.0.1:10000";
const MIN_FLOOR: u8 = 1;
const MAX_FLOOR: u8 = 5;
const MIN_KEY: u8 = 0;
const MAX_KEY: u8 = 3;

pub struct ElevatorApp {
    socket: Arc<UdpSocket>,
}

impl ElevatorApp {
    pub async fn new() -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(CONTROL_ADDRESS).await?);
        println!("Listening on {CONTROL_ADDRESS}");
        Ok(Self { socket })
    }

    pub async fn run(self) -> Result<()> {
        // Initialize the channel and state
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Command>();
        let state = Arc::new(Mutex::new(SchedulerState {
            current_floor: MIN_FLOOR,
            direction_up: true,
            ..SchedulerState::default()
        }));

        let prestart = ElevatorState::<PreStart>::new(state.clone(), tx);
        let init = prestart.init().await?;
        println!("Elevator controller initialized");

        let boxed = init.boxed();
        let state_machine = Arc::new(Mutex::new(Some(boxed)));
        let scheduler_strategy = ScanStrategy::new(state.clone());
        let scheduler = SchedulerEventLayer::new(scheduler_strategy, state_machine.clone());
        let controller_service = ControllerService::new(state_machine);

        let shared_socket = self.socket.clone();
        controller_service
            .run_background(shared_socket, rx, LIFTY_ADDRESS)
            .await?;

        let mut svc = ServiceBuilder::new()
            .layer(UdpEventLayer)
            .layer(scheduler)
            .service(controller_service);

        let mut buf = vec![0u8; UDP_MAX_SIZE];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            println!("Got UDP packet from {}", addr);

            let raw = &buf[..len];
            svc.ready().await?;
            if svc.call(raw).await.is_err() {
                eprintln!("Service error");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = ElevatorApp::new().await?;
    app.run().await
}
