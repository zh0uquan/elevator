use elevator::{Command, event, my_event_handler};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tower::{Service, ServiceBuilder, ServiceExt, service_fn};

const UDP_MAX_SIZE: usize = 65535;

const CONTROL_ADDRESS: &str = "127.0.0.1:11000";
const LIFTY_ADDRESS: &str = "127.0.0.1:10000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind(CONTROL_ADDRESS).await?;
    println!("Listening on {}", CONTROL_ADDRESS);
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
    let mut svc = ServiceBuilder::new()
        .layer(event::UdpEventLayer)
        .service(handler_svc);

    loop {
        let (len, addr) = shared_socket.recv_from(&mut buf).await?;
        println!("Got connected from {}", addr);

        let raw = &buf[..len];
        svc.ready().await.unwrap();
        if let Err(_) = svc.call(raw).await {
            eprintln!("Service error");
        }
    }
}
