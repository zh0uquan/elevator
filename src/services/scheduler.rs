use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use tower::{Layer, Service};

use crate::strategy::Strategy;
use crate::types::event::Event;
use crate::types::sched_events::{Action, ScheduleEvent};

pub struct SchedulerService<S, ST> {
    inner: Arc<Mutex<S>>,
    strategy: ST,
}

impl<S, ST> SchedulerService<S, ST> {
    fn new(inner: S, strategy: ST) -> Self {
        SchedulerService {
            inner: Arc::new(Mutex::new(inner)),
            strategy,
        }
    }
}

pub struct SchedulerEventLayer<ST> {
    strategy: ST,
}

impl<ST> SchedulerEventLayer<ST> {
    pub fn new(strategy: ST) -> Self {
        Self { strategy }
    }
}

impl<S, ST> Layer<S> for SchedulerEventLayer<ST>
where
    ST: Clone,
{
    type Service = SchedulerService<S, ST>;
    fn layer(&self, inner: S) -> Self::Service {
        SchedulerService::new(inner, self.strategy.clone())
    }
}

impl<S, ST> Service<Event> for SchedulerService<S, ST>
where
    S: Service<Action, Response = (), Error = anyhow::Error> + Send + 'static,
    S::Future: Send + 'static,
    ST: Clone + Strategy<Event, ScheduleEvent> + Send + 'static,
{
    type Response = ();
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Event) -> Self::Future {
        let inner = self.inner.clone();
        let strategy = self.strategy.clone();

        Box::pin(async move {
            strategy.handle(event).await;
            let maybe_sched_events = strategy.step().await;
            if let Some(mut schedule_event) = maybe_sched_events {
                while let Some(event) = schedule_event.pop_front() {
                    match event {
                        ScheduleEvent::Instant(action) => {
                            inner.lock().await.call(action).await?;
                        }
                        ScheduleEvent::WaitTime(duration, action) => {
                            tokio::time::sleep(duration).await;
                            inner.lock().await.call(action).await?;
                        }
                    }
                }
            } else {
                println!("No action generated");
            }
            Ok(())
        })
    }
}
