use anyhow::Context;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tokio::sync::Mutex;
use tower::{Layer, Service};

#[derive(Debug)]
pub enum Event {
    ElevatorUp(u8),
    ElevatorDown(u8),
    PanelButtonPressed(u8),
    ElevatorApproaching(u8),
    ElevatorStopped(u8),
    DoorOpened(u8),
    DoorClosed(u8),
    KeySwitched(u8),
}

impl TryFrom<&[u8]> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> anyhow::Result<Self> {
        if value.len() < 2 {
            anyhow::bail!(
                "buffer too short: expected at least 2 bytes, got {}",
                value.len()
            );
        }

        let cmd = value[0];
        let num_str = str::from_utf8(&value[1..]).context("argument is not valid UTF‑8")?;
        let arg = num_str
            .parse::<u8>()
            .context(format!("failed to parse '{num_str}' as u8"))?;

        match cmd {
            b'U' => Ok(Event::ElevatorUp(arg)),
            b'D' => Ok(Event::ElevatorDown(arg)),
            b'P' => Ok(Event::PanelButtonPressed(arg)),
            b'A' => Ok(Event::ElevatorApproaching(arg)),
            b'S' => Ok(Event::ElevatorStopped(arg)),
            b'O' => Ok(Event::DoorOpened(arg)),
            b'C' => Ok(Event::DoorClosed(arg)),
            b'K' => Ok(Event::KeySwitched(arg)),
            other => anyhow::bail!("unknown event byte: {}", other),
        }
    }
}

pub struct UdpEventService<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> UdpEventService<S> {
    fn new(inner: S) -> Self {
        UdpEventService {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<S> Service<&[u8]> for UdpEventService<S>
where
    S: Service<Event, Response = (), Error = ()> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = ();
    type Error = ();
    type Future = Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, raw: &[u8]) -> Self::Future {
        let maybe_event = Event::try_from(raw);
        let inner = self.inner.clone();

        Box::pin(async move {
            match maybe_event {
                Ok(ev) => inner.lock().await.call(ev).await,
                Err(e) => {
                    eprintln!("Invalid packet: {e:?}");
                    Ok(())
                }
            }
        })
    }
}

pub struct UdpEventLayer;

impl<S> Layer<S> for UdpEventLayer {
    type Service = UdpEventService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UdpEventService::new(inner)
    }
}
