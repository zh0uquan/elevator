use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait Strategy<Event, ScheduleEvent>: Send + Sync {
    async fn handle(&self, event: Event);
    async fn step(&self) -> Option<VecDeque<ScheduleEvent>>;
}
