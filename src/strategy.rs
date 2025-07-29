use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait Strategy<Event, ScheduleEvent, StateMachine>: Send + Sync {
    async fn handle(&self, event: Event);
    async fn step(&self) -> Option<VecDeque<ScheduleEvent>>;

    async fn recommend(&self, state_machine: &StateMachine);
}
