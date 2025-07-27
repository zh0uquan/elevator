use crate::ScheduleEvent;
use crate::transition::BoxedTransition;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use tokio::sync::Mutex;
use tower::Service;

pub struct ControllerService {
    transition: Arc<Mutex<Option<BoxedTransition>>>,
}

impl ControllerService {
    pub fn new(transition: BoxedTransition) -> Self {
        ControllerService {
            transition: Arc::new(Mutex::new(Some(transition))),
        }
    }
}

impl Service<ScheduleEvent> for ControllerService {
    type Response = ();
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: ScheduleEvent) -> Self::Future {
        let transition = Arc::clone(&self.transition);
        Box::pin(async move {
            let mut guard = transition.lock().await;
            let current = guard
                .take()
                .ok_or_else(|| anyhow::anyhow!("Transition was None"))?;
            let next = current.on_event(event).await?;
            *guard = Some(next);
            Ok(())
        })
    }
}
