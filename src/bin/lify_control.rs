use elevator::strategy::SchedulerState;
use elevator::transition::{ElevatorState, PreStart};
use elevator::{Command, Event, controller, event, scheduler, strategy};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tower::filter::Predicate;
use tower::{BoxError, Service, ServiceBuilder, ServiceExt};

const UDP_MAX_SIZE: usize = 65535;

const CONTROL_ADDRESS: &str = "127.0.0.1:11000";
const LIFTY_ADDRESS: &str = "127.0.0.1:10000";

const MIN_FLOOR: u8 = 1;
const MAX_FLOOR: u8 = 5;

const MIN_KEY: u8 = 0;
const MAX_KEY: u8 = 3;

#[derive(Clone)]
struct Validation;

impl Predicate<Event> for Validation {
    type Request = Event;

    fn check(&mut self, event: Event) -> Result<Self::Request, BoxError> {
        let valid = match event {
            Event::ElevatorUp(f)
            | Event::ElevatorDown(f)
            | Event::ElevatorApproaching(f)
            | Event::DoorOpened(f)
            | Event::DoorClosed(f)
            | Event::ElevatorStopped(f)
            | Event::PanelButtonPressed(f) => (MIN_FLOOR..=MAX_FLOOR).contains(&f),
            Event::KeySwitched(k) => k <= MAX_KEY,
        };
        if !valid {
            eprintln!("invalid event: {event:?}");
            return Err(BoxError::from("invalid event"));
        }
        Ok(event)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind(CONTROL_ADDRESS).await?;
    println!("Listening on {CONTROL_ADDRESS}");
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

    let shared_socket = Arc::new(socket);
    let shared_socket_clone = shared_socket.clone();

    let mut buf = vec![0u8; UDP_MAX_SIZE];

    let state = Arc::new(Mutex::new(SchedulerState {
        current_floor: MIN_FLOOR,
        direction_up: true,
        ..SchedulerState::default()
    }));
    let prestart = ElevatorState::<PreStart>::new(state.clone(), tx);
    let init = prestart.init().await?;
    println!("Elevator controller initialized");
    let scheduler_strategy = strategy::ScanStrategy::new(state.clone());
    let scheduler = scheduler::SchedulerEventLayer::new(scheduler_strategy);
    let controller_service = controller::ControllerService::new(Box::new(init));
    controller_service
        .run_background(shared_socket_clone, rx, LIFTY_ADDRESS)
        .await?;
    let mut svc = ServiceBuilder::new()
        .layer(event::UdpEventLayer)
        .layer(scheduler)
        .service(controller_service);

    loop {
        let (len, addr) = shared_socket.recv_from(&mut buf).await?;
        println!("Got udp packet from {addr}");

        let raw = &buf[..len];
        svc.ready().await?;
        if svc.call(raw).await.is_err() {
            eprintln!("Service error");
        }
    }
}
