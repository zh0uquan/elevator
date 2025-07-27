pub mod controller;
pub mod event;
pub mod scheduler;
pub mod strategy;
pub mod transition;

use anyhow::Context;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum Event {
    ElevatorUp(u8),
    ElevatorDown(u8),
    PanelButtonPressed(u8),
    ElevatorApproaching(u8),
    ElevatorStopped(u8),
    DoorOpened(u8),
    DoorClosed(u8),
    KeySwitched(u8),
}

impl TryFrom<&[u8]> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> anyhow::Result<Self> {
        if value.len() < 2 {
            anyhow::bail!(
                "buffer too short: expected at least 2 bytes, got {}",
                value.len()
            );
        }

        let cmd = value[0];
        let num_str = str::from_utf8(&value[1..]).context("argument is not valid UTF‑8")?;
        let arg = num_str
            .parse::<u8>()
            .context(format!("failed to parse '{num_str}' as u8"))?;

        match cmd {
            b'U' => Ok(Event::ElevatorUp(arg)),
            b'D' => Ok(Event::ElevatorDown(arg)),
            b'P' => Ok(Event::PanelButtonPressed(arg)),
            b'A' => Ok(Event::ElevatorApproaching(arg)),
            b'S' => Ok(Event::ElevatorStopped(arg)),
            b'O' => Ok(Event::DoorOpened(arg)),
            b'C' => Ok(Event::DoorClosed(arg)),
            b'K' => Ok(Event::KeySwitched(arg)),
            other => anyhow::bail!("unknown event byte: {}", other),
        }
    }
}

#[async_trait]
pub trait Strategy<Event, ScheduleEvent>: Send + Sync {
    async fn handle(&self, event: Event);
    async fn step(&self) -> Option<VecDeque<ScheduleEvent>>;
}

#[derive(Debug)]
pub enum ScheduleEvent {
    Moving(u8),
    Braking,
    Stopped,
    OpeningDoor,
    ClosingDoor,
    DoorOpened,
    DoorClosed,
}

pub enum Command {
    MU,
    MD,
    S,
    DO,
    DC,
    CP(u8),
    CU(u8),
    CD(u8),
    IU(u8),
    ID(u8),
    CI(u8),
    R,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::MU => write!(f, "MU"),
            Command::MD => write!(f, "MD"),
            Command::S => write!(f, "S"),
            Command::DO => write!(f, "DO"),
            Command::DC => write!(f, "DC"),
            Command::R => write!(f, "R"),
            Command::CP(v) => write!(f, "CP{v}"),
            Command::CU(v) => write!(f, "CU{v}"),
            Command::CD(v) => write!(f, "CD{v}"),
            Command::IU(v) => write!(f, "IU{v}"),
            Command::ID(v) => write!(f, "ID{v}"),
            Command::CI(v) => write!(f, "CI{v}"),
        }
    }
}
