use crate::elevator::ElevatorData;
use crate::strategy::Strategy;
use crate::transition::{SharedStateMachine, State};
use crate::types::event::Event;
use crate::types::sched_events::{Action, ScheduleEvent};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ScanStrategy {
    elevator_data: Arc<Mutex<ElevatorData>>,
}

impl ScanStrategy {
    pub fn new(elevator_data: Arc<Mutex<ElevatorData>>) -> Self {
        Self { elevator_data }
    }
}

#[async_trait]
impl Strategy<Event, ScheduleEvent, SharedStateMachine> for ScanStrategy {
    async fn handle(
        &self,
        event: Event,
        state_machine: &SharedStateMachine,
    ) -> Option<VecDeque<ScheduleEvent>> {
        let mut elevator_data = self.elevator_data.lock().await;
        let state = state_machine
            .lock()
            .await
            .as_ref()
            .expect("state machine should not be None")
            .state();
        let mut sched_events = VecDeque::new();
        match event {
            Event::PanelButtonPressed(floor)
            | Event::ElevatorUp(floor)
            | Event::ElevatorDown(floor) => {
                elevator_data.enqueue_request(floor);
            }
            Event::DoorOpened(floor) => {
                if elevator_data.active_target == Some(floor) && state == State::DoorOpening {
                    sched_events.push_back(ScheduleEvent::Instant(Action::DoorOpened));
                    sched_events.push_back(ScheduleEvent::WaitTime(
                        Duration::from_secs(2),
                        Action::ClosingDoor,
                    ));
                } else {
                    eprintln!("elevator behaving strange, door opened on unexpected floor: {floor}")
                }
            }
            Event::DoorClosed(floor) => {
                if state == State::DoorClosing {
                    sched_events.push_back(ScheduleEvent::Instant(Action::DoorClosed))
                } else {
                    eprintln!("elevator behaving strange, door closed on unexpected floor: {floor}")
                }
            }
            Event::ElevatorStopped(floor) => {
                sched_events.push_back(ScheduleEvent::Instant(Action::Stopped));
                if elevator_data.active_target == Some(floor) && state == State::Braking {
                    elevator_data.current_floor = floor;
                    sched_events.push_back(ScheduleEvent::Instant(Action::Stopped));
                    sched_events.push_back(ScheduleEvent::Instant(Action::OpeningDoor))
                } else {
                    eprintln!(
                        "elevator behaving strange, door stopped on unexpected floor: {floor}"
                    )
                }
            }
            Event::ElevatorApproaching(floor) => {
                if elevator_data.active_target == Some(floor)
                    && (state == State::MovingUp || state == State::MovingDown)
                {
                    sched_events.push_back(ScheduleEvent::Instant(Action::Braking))
                } else {
                    println!("elevator approaching floor: {floor}")
                }
            }
            Event::KeySwitched(floor) => {}
        }

        println!("{:?} with state {:?}", elevator_data, state);

        if state == State::Idle || matches!(event, Event::DoorClosed(_)) {
            if let Some(target) = elevator_data.next_target() {
                if target > elevator_data.current_floor {
                    sched_events.push_back(ScheduleEvent::Instant(Action::MovingUp));
                } else if target < elevator_data.current_floor {
                    sched_events.push_back(ScheduleEvent::Instant(Action::MovingDown));
                } else {
                    println!("elevator already on floor {target}");
                }
            }
        }

        (!sched_events.is_empty()).then_some(sched_events)
    }
}
