use async_trait::async_trait;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::strategy::Strategy;
use crate::types::event::Event;
use crate::types::sched_events::{Action, ScheduleEvent};

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum Status {
    #[default]
    Idle,
    Braking,
    Stopped,
    Moving,
    DoorOpening,
    DoorOpened,
    DoorClosed,
    DoorClosing,
}

#[derive(Debug, Default)]
pub struct SchedulerState {
    pub current_floor: u8,
    pub direction_up: bool,
    pub up_queue: BinaryHeap<Reverse<u8>>,
    pub down_queue: BinaryHeap<u8>,
    pub active_target: Option<u8>,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct ScanStrategy {
    state: Arc<Mutex<SchedulerState>>,
}

impl ScanStrategy {
    pub fn new(state: Arc<Mutex<SchedulerState>>) -> Self {
        Self { state }
    }

    fn add_to_queue(state: &mut SchedulerState, floor: u8) {
        if floor > state.current_floor {
            state.up_queue.push(Reverse(floor));
        } else if floor < state.current_floor {
            state.down_queue.push(floor);
        }
    }

    fn next_floor(state: &mut SchedulerState) -> Option<u8> {
        if state.direction_up {
            state.up_queue.pop().map(|Reverse(f)| f)
        } else {
            state.down_queue.pop()
        }
    }

    fn try_next_floor(state: &mut SchedulerState) -> Option<u8> {
        if let Some(floor) = Self::next_floor(state) {
            return Some(floor);
        }
        state.direction_up = !state.direction_up;
        Self::next_floor(state)
    }
}

#[async_trait]
impl Strategy<Event, ScheduleEvent> for ScanStrategy {
    async fn handle(&self, event: Event) {
        let mut state = self.state.lock().await;
        match event {
            Event::PanelButtonPressed(floor)
            | Event::ElevatorUp(floor)
            | Event::ElevatorDown(floor) => {
                Self::add_to_queue(&mut state, floor);
                if state.status == Status::Idle {
                    state.status = Status::Moving;
                }
            }
            Event::DoorOpened(floor) => {
                if state.active_target == Some(floor) {
                    state.status = Status::DoorOpened;
                } else {
                    eprintln!(
                        "Unexpected door opened on floor {floor}, target: {:?}",
                        state.active_target
                    );
                }
            }
            Event::DoorClosed(floor) => {
                if state.active_target == Some(floor) {
                    state.status = Status::DoorClosed;
                    state.active_target = None;
                } else {
                    eprintln!(
                        "Unexpected door closed on floor {floor}, target: {:?}",
                        state.active_target
                    );
                }
            }
            Event::ElevatorApproaching(floor) => {
                if state.active_target == Some(floor) {
                    state.status = Status::Braking;
                } else {
                    println!("Approaching floor {floor}, no active target");
                }
            }
            Event::ElevatorStopped(floor) => {
                state.status = Status::Stopped;
                state.current_floor = floor;
            }
            Event::KeySwitched(_) => {
                println!("Key switched event received");
            }
        }
    }

    async fn step(&self) -> Option<VecDeque<ScheduleEvent>> {
        let mut state = self.state.lock().await;
        let mut events = VecDeque::new();

        match state.status {
            Status::Idle => {
                if let Some(floor) = Self::try_next_floor(&mut state) {
                    state.active_target = Some(floor);
                    state.status = Status::Moving;
                    events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                }
            }
            Status::Moving => {
                if state.active_target.is_none() {
                    if let Some(floor) = Self::try_next_floor(&mut state) {
                        state.active_target = Some(floor);
                        events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                    } else {
                        state.status = Status::Idle;
                    }
                }
            }
            Status::Braking => {
                events.push_back(ScheduleEvent::Instant(Action::Braking));
            }
            Status::Stopped => {
                events.push_back(ScheduleEvent::Instant(Action::Stopped));
                if state.active_target == Some(state.current_floor) {
                    events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
                    state.status = Status::DoorOpening;
                }
            }
            Status::DoorOpening => {
                events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
            }
            Status::DoorOpened => {
                events.push_back(ScheduleEvent::Instant(Action::DoorOpened));
                events.push_back(ScheduleEvent::WaitTime(
                    Duration::from_secs(2),
                    Action::ClosingDoor,
                ));
                state.status = Status::DoorClosing;
            }
            Status::DoorClosing => {
                events.push_back(ScheduleEvent::Instant(Action::ClosingDoor));
            }
            Status::DoorClosed => {
                events.push_back(ScheduleEvent::Instant(Action::DoorClosed));
                if let Some(floor) = Self::try_next_floor(&mut state) {
                    state.active_target = Some(floor);
                    state.status = Status::Moving;
                    events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                } else {
                    state.status = Status::Idle;
                }
            }
        }

        (!events.is_empty()).then_some(events)
    }
}
