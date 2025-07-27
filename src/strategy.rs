use crate::{Action, Event};
use crate::{ScheduleEvent, Strategy};
use async_trait::async_trait;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
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

    fn next(state: &mut SchedulerState) -> Option<u8> {
        if state.direction_up {
            state.up_queue.pop().map(|Reverse(f)| f)
        } else {
            state.down_queue.pop()
        }
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
                if floor > state.current_floor {
                    state.up_queue.push(Reverse(floor));
                    state.status = Status::Moving;
                } else if floor < state.current_floor {
                    state.down_queue.push(floor);
                    state.status = Status::Moving;
                } else {
                    println!(
                        "Floor already on {:?}, button pressed on {:?}",
                        state.current_floor, floor
                    );
                }
            }
            Event::DoorOpened(floor) => {
                if let Some(target) = state.active_target {
                    if target == floor {
                        state.status = Status::DoorOpened;
                    }
                } else {
                    eprintln!(
                        "Door Opened on {floor}, target is {:?}",
                        state.active_target
                    );
                }
            }
            Event::DoorClosed(floor) => {
                if let Some(target) = state.active_target {
                    if target == floor {
                        state.status = Status::DoorClosed;
                        state.active_target = None;
                    }
                } else {
                    eprintln!(
                        "Door Closed on {floor}, target is {:?}",
                        state.active_target
                    );
                }
            }
            Event::ElevatorApproaching(floor) => {
                if let Some(target) = state.active_target {
                    if target == floor {
                        state.status = Status::Braking;
                    }
                } else {
                    println!(
                        "Approaching to {floor}, target is {:?}",
                        state.active_target
                    );
                }
            }
            Event::ElevatorStopped(floor) => {
                state.status = Status::Stopped;
                state.current_floor = floor;
                println!("Stopped to {floor}");
            }
            Event::KeySwitched(_) => {
                println!("key switched");
            }
        }
    }

    async fn step(&self) -> Option<VecDeque<ScheduleEvent>> {
        let mut state = self.state.lock().await;
        let mut schedule_events = VecDeque::new();
        match state.status {
            Status::Braking => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::Braking));
            }
            Status::Stopped => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::Stopped));
                if let Some(target) = state.active_target {
                    if target == state.current_floor {
                        schedule_events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
                    }
                }
            }
            Status::Moving | Status::Idle => {
                if state.active_target.is_some() {
                    return None;
                }
                // Try the current direction
                if let Some(floor) = Self::next(&mut state) {
                    state.active_target = Some(floor);
                    schedule_events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                    return Some(schedule_events);
                }
                // Flip direction and try again
                state.direction_up = !state.direction_up;
                if let Some(floor) = Self::next(&mut state) {
                    state.active_target = Some(floor);
                    schedule_events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                    return Some(schedule_events);
                }
            }
            Status::DoorOpening => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::OpeningDoor));
            }
            Status::DoorOpened => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::DoorOpened));
                schedule_events.push_back(ScheduleEvent::WaitTime(
                    Duration::from_secs(2),
                    Action::ClosingDoor,
                ));
            }
            Status::DoorClosed => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::DoorClosed));
                if let Some(floor) = Self::next(&mut state) {
                    state.active_target = Some(floor);
                    schedule_events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                    return Some(schedule_events);
                }
                // Flip direction and try again
                state.direction_up = !state.direction_up;
                if let Some(floor) = Self::next(&mut state) {
                    state.active_target = Some(floor);
                    schedule_events.push_back(ScheduleEvent::Instant(Action::Moving(floor)));
                    return Some(schedule_events);
                }
            }
            Status::DoorClosing => {
                schedule_events.push_back(ScheduleEvent::Instant(Action::ClosingDoor));
            }
        }
        if schedule_events.is_empty() {
            None
        } else {
            Some(schedule_events)
        }
    }
}
