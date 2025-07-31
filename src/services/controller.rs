use crate::transition::BoxedTransition;
use crate::types::cmd::Command;
use crate::types::sched_events::Action;

use crate::context::ElevatorContext;
use futures::ready;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tower::Service;

pub struct ControllerService {
    is_ready: Arc<Mutex<bool>>,
    transition: Arc<Mutex<Option<BoxedTransition>>>,
    elevator_context: Arc<Mutex<ElevatorContext>>,
}

impl ControllerService {
    pub fn new(
        transition: Arc<Mutex<Option<BoxedTransition>>>,
        elevator_context: Arc<Mutex<ElevatorContext>>,
    ) -> Self {
        ControllerService {
            is_ready: Arc::new(Mutex::new(false)),
            transition,
            elevator_context,
        }
    }

    pub async fn run_background(
        &self,
        socket: Arc<UdpSocket>,
        mut rx: tokio::sync::mpsc::UnboundedReceiver<Command>,
        address: &'static str,
    ) -> anyhow::Result<()> {
        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                socket
                    .send_to(cmd.to_string().as_bytes(), address)
                    .await
                    .expect("failed to send command");
            }
        });
        *self.is_ready.lock().await = true;
        Ok(())
    }
}

impl Service<Action> for ControllerService {
    type Response = ();
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut lock_fut = Box::pin(self.is_ready.lock());
        let guard = ready!(lock_fut.as_mut().poll(cx));
        if !*guard {
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, action: Action) -> Self::Future {
        let transition = Arc::clone(&self.transition);
        let elevator_context = Arc::clone(&self.elevator_context);
        Box::pin(async move {
            let mut guard = transition.lock().await;
            let current = guard
                .take()
                .ok_or_else(|| anyhow::anyhow!("Transition was None"))?;
            let mut ctx = elevator_context.lock().await;
            let next = current.on_event(action, &mut ctx).await?;
            *guard = Some(next);
            Ok(())
        })
    }
}
