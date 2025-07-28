use std::time::Duration;

#[derive(Debug)]
pub enum ScheduleEvent {
    Instant(Action),
    WaitTime(Duration, Action),
}

#[derive(Debug)]
pub enum Action {
    Moving(u8),
    Braking,
    Stopped,
    OpeningDoor,
    ClosingDoor,
    DoorOpened,
    DoorClosed,
}
