use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Default)]
pub struct ElevatorData {
    pub current_floor: u8,
    pub direction_up: bool,
    pub up_queue: BinaryHeap<Reverse<u8>>,
    pub down_queue: BinaryHeap<u8>,
    pub active_target: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct ScanStrategy {}

impl ElevatorData {
    pub fn enqueue_request(&mut self, floor: u8) {
        if floor > self.current_floor {
            self.up_queue.push(Reverse(floor));
        } else if floor < self.current_floor {
            self.down_queue.push(floor);
        }
    }

    fn next_target_in_direction(&mut self) -> Option<u8> {
        let next_target = if self.direction_up {
            self.up_queue.pop().map(|Reverse(f)| f)
        } else {
            self.down_queue.pop()
        };
        self.active_target = next_target;
        next_target
    }

    pub fn next_target(&mut self) -> Option<u8> {
        if let Some(floor) = self.next_target_in_direction() {
            return Some(floor);
        }
        self.direction_up = !self.direction_up;
        self.next_target_in_direction()
    }
}
