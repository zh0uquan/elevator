use std::time::Duration;

#[derive(Debug)]
pub enum ScheduleEvent {
    Instant(Action),
    WaitTime(Duration, Action),
}

#[derive(Debug)]
pub enum Action {
    MovingUp,
    MovingDown,
    Braking,
    Stopped,
    OpeningDoor,
    ClosingDoor,
    DoorOpened,
    DoorClosed,
}
