use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    AtFloor(u8),
    BetweenFloors(u8, u8),
}

impl Default for Location {
    fn default() -> Self {
        Location::AtFloor(0)
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Location::*;

        fn midpoint(from: u8, to: u8) -> f32 {
            (from as f32 + to as f32) / 2.0
        }

        let lhs = match self {
            AtFloor(f) => *f as f32,
            BetweenFloors(f1, f2) => midpoint(*f1, *f2),
        };

        let rhs = match other {
            AtFloor(f) => *f as f32,
            BetweenFloors(f1, f2) => midpoint(*f1, *f2),
        };

        lhs.partial_cmp(&rhs)
    }
}

#[derive(Debug, Default)]
pub struct ElevatorContext {
    pub current_location: Location,
    pub direction_up: bool,
    pub up_queue: BinaryHeap<Reverse<u8>>,
    pub down_queue: BinaryHeap<u8>,
    pub active_target: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct ScanStrategy {}

impl ElevatorContext {
    pub fn move_to(&mut self, location: Location) {
        self.current_location = location;
    }

    pub fn enqueue_request(&mut self, floor: u8) {
        if Location::AtFloor(floor) > self.current_location {
            self.up_queue.push(Reverse(floor));
        } else if Location::AtFloor(floor) < self.current_location {
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
