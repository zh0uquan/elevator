use crate::Event;
use crate::{ScheduleEvent, Strategy};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
struct SchedulerState {
    current_floor: u8,
    direction_up: bool,
    up_queue: BinaryHeap<Reverse<u8>>,
    down_queue: BinaryHeap<u8>,
    active_target: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct ScanStrategy {
    state: Arc<Mutex<SchedulerState>>,
}

impl ScanStrategy {
    pub fn new(start_floor: u8, direction_up: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState {
                current_floor: start_floor,
                direction_up,
                ..SchedulerState::default()
            })),
        }
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
                } else if floor < state.current_floor {
                    state.down_queue.push(floor);
                }
            }
            Event::DoorOpened(u8) => {}
            Event::DoorClosed(u8) => {}
            Event::ElevatorApproaching(u8) => {}
            Event::ElevatorStopped(u8) => {}
            Event::KeySwitched(_) => {
                println!("key switched");
            }
        }
    }

    async fn step(&self) -> Option<ScheduleEvent> {
        let mut state = self.state.lock().await;
        if state.active_target.is_some() {
            return None;
        }
        // Try the current direction
        if let Some(floor) = Self::next(&mut state) {
            state.active_target = Some(floor);
            return Some(ScheduleEvent::Goto(floor));
        }

        // Flip direction and try again
        state.direction_up = !state.direction_up;
        if let Some(floor) = Self::next(&mut state) {
            state.active_target = Some(floor);
            return Some(ScheduleEvent::Goto(floor));
        }
        None
    }
}
