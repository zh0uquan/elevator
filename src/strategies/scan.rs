use crate::elevator::ElevatorData;
use crate::strategy::Strategy;
use crate::transition::SharedStateMachine;
use crate::types::event::Event;
use crate::types::sched_events::ScheduleEvent;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
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
        let elevator_data = self.elevator_data.lock().await;

        None
    }
    //     let mut state = self.state.lock().await;
    //     match event {
    //         Event::PanelButtonPressed(floor)
    //         | Event::ElevatorUp(floor)
    //         | Event::ElevatorDown(floor) => {
    //             Self::add_to_queue(&mut state, floor);
    //             if state.status == Status::Idle {
    //                 state.status = Status::Moving;
    //             }
    //         }
    //         Event::DoorOpened(floor) => {
    //             if state.active_target == Some(floor) {
    //                 state.status = Status::DoorOpened;
    //             } else {
    //                 eprintln!(
    //                     "Unexpected door opened on floor {floor}, target: {:?}",
    //                     state.active_target
    //                 );
    //             }
    //         }
    //         Event::DoorClosed(floor) => {
    //             if state.active_target == Some(floor) {
    //                 state.status = Status::DoorClosed;
    //                 state.active_target = None;
    //             } else {
    //                 eprintln!(
    //                     "Unexpected door closed on floor {floor}, target: {:?}",
    //                     state.active_target
    //                 );
    //             }
    //         }
    //         Event::ElevatorApproaching(floor) => {
    //             if state.active_target == Some(floor) {
    //                 state.status = Status::Braking;
    //             } else {
    //                 println!("Approaching floor {floor}, no active target");
    //             }
    //         }
    //         Event::ElevatorStopped(floor) => {
    //             state.status = Status::Stopped;
    //             state.current_floor = floor;
    //         }
    //         Event::KeySwitched(_) => {
    //             println!("Key switched event received");
    //         }
    //     }
    // }
    //
    // async fn step(&self) -> Option<VecDeque<ScheduleEvent>> {
    //     let mut state = self.state.lock().await;
    //     let mut events = VecDeque::new();
    //
    //     match state.status {
    //         Status::Idle => {
    //             if let Some(floor) = Self::try_next_floor(&mut state) {
    //                 state.active_target = Some(floor);
    //                 state.status = Status::Moving;
    //                 events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
    //             }
    //         }
    //         Status::Moving => {
    //             if state.active_target.is_none() {
    //                 if let Some(floor) = Self::try_next_floor(&mut state) {
    //                     state.active_target = Some(floor);
    //                     events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
    //                 } else {
    //                     state.status = Status::Idle;
    //                 }
    //             }
    //         }
    //         Status::Braking => {
    //             events.push_back(ScheduleEvent::Instant(Action::Braking));
    //         }
    //         Status::Stopped => {
    //             events.push_back(ScheduleEvent::Instant(Action::Stopped));
    //             if state.active_target == Some(state.current_floor) {
    //                 events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
    //                 state.status = Status::DoorOpening;
    //             }
    //         }
    //         Status::DoorOpening => {
    //             events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
    //         }
    //         Status::DoorOpened => {
    //             events.push_back(ScheduleEvent::Instant(Action::DoorOpened));
    //             events.push_back(ScheduleEvent::WaitTime(
    //                 Duration::from_secs(2),
    //                 Action::ClosingDoor,
    //             ));
    //             state.status = Status::DoorClosing;
    //         }
    //         Status::DoorClosing => {
    //             events.push_back(ScheduleEvent::Instant(Action::ClosingDoor));
    //         }
    //         Status::DoorClosed => {
    //             events.push_back(ScheduleEvent::Instant(Action::DoorClosed));
    //             if let Some(floor) = Self::try_next_floor(&mut state) {
    //                 state.active_target = Some(floor);
    //                 state.status = Status::Moving;
    //                 events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
    //             } else {
    //                 state.status = Status::Idle;
    //             }
    //         }
    //     }
    //
    //     (!events.is_empty()).then_some(events)
    // }
    //
}
