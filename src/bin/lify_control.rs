use elevator::event::Event;
use elevator::{Command, event, my_event_handler, scheduler, strategy};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tower::filter::Predicate;
use tower::{BoxError, Service, ServiceBuilder, ServiceExt, service_fn};

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
            event::Event::ElevatorUp(f)
            | event::Event::ElevatorDown(f)
            | event::Event::ElevatorApproaching(f)
            | event::Event::DoorOpened(f)
            | event::Event::DoorClosed(f)
            | event::Event::ElevatorStopped(f)
            | event::Event::PanelButtonPressed(f) => (MIN_FLOOR..=MAX_FLOOR).contains(&f),
            event::Event::KeySwitched(k) => k <= MAX_KEY,
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
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

    let shared_socket = Arc::new(socket);
    let shared_socket_clone = shared_socket.clone();
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            shared_socket_clone
                .send_to(cmd.to_string().as_bytes(), LIFTY_ADDRESS)
                .await
                .expect("failed to send command");
        }
    });

    let mut buf = vec![0u8; UDP_MAX_SIZE];
    let handler_svc = service_fn(my_event_handler);
    // let prestart_controller = ElevatorController::new(tx);
    // let mut controller = prestart_controller.init().await?;
    // println!("Elevator controller initialized");
    // let validation = Validation;

    let scheduler_strategy = strategy::ScanStrategy::new(1, true);
    let scheduler = scheduler::SchedulerEventLayer::new(scheduler_strategy);
    let mut svc = ServiceBuilder::new()
        .layer(event::UdpEventLayer)
        .layer(scheduler)
        .service(handler_svc);

    loop {
        let (len, addr) = shared_socket.recv_from(&mut buf).await?;
        println!("Got connected from {addr}");

        let raw = &buf[..len];
        svc.ready().await.unwrap();
        if svc.call(raw).await.is_err() {
            eprintln!("Service error");
        }
    }
}
