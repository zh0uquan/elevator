use anyhow::Context;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
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
