use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait Strategy<Event, ScheduleEvent, StateMachine>: Send + Sync {
    async fn handle(
        &self,
        event: Event,
        state_machine: &StateMachine,
    ) -> Option<VecDeque<ScheduleEvent>>;
}
