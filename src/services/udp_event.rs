use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tokio::sync::Mutex;
use tower::{Layer, Service};

use crate::types::event::Event;

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
    S: Service<Event, Response = (), Error = anyhow::Error> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = ();
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, raw: &[u8]) -> Self::Future {
        let maybe_event = Event::try_from(raw);
        let inner = self.inner.clone();

        Box::pin(async move {
            match maybe_event {
                Ok(ev) => {
                    println!("Event received: {ev:?}");
                    inner.lock().await.call(ev).await
                }
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
