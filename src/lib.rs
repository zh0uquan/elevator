pub mod event;
mod scheduler;

use crate::event::Event;
use async_trait::async_trait;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

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
            Command::CP(v) => write!(f, "CP{}", v),
            Command::CU(v) => write!(f, "CU{}", v),
            Command::CD(v) => write!(f, "CD{}", v),
            Command::IU(v) => write!(f, "IU{}", v),
            Command::ID(v) => write!(f, "ID{}", v),
            Command::CI(v) => write!(f, "CI{}", v),
        }
    }
}

#[async_trait]
pub trait Transition: Send + 'static {
    async fn on_event(self: Box<Self>, event: Event) -> Box<dyn Transition + Send + 'static>;
}

#[derive(Debug)]
struct ElevatorState {
    current_floor: u8,
    target_floor: Option<u8>,
}

#[derive(Debug)]
struct ElevatorController<State> {
    state: ElevatorState,
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
    _marker: PhantomData<State>,
}

impl<State> ElevatorController<State> {
    async fn send_command(&self, command: Command) -> anyhow::Result<()> {
        self.tx.send(command)?;
        Ok(())
    }
}

pub struct PreStart;
pub struct Idle;
pub struct MovingUp;
pub struct MovingDown;
pub struct DoorOpening;
pub struct DoorOpen;
pub struct DoorClosing;
pub struct Braking;
pub struct EmergencyBrake;

impl ElevatorController<PreStart> {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<Command>) -> ElevatorController<PreStart> {
        ElevatorController::<PreStart> {
            state: ElevatorState {
                current_floor: 1, // this is same as lifty, maybe we need to make this
                target_floor: None,
            },
            tx,
            _marker: PhantomData,
        }
    }
    async fn init(self) -> anyhow::Result<ElevatorController<Idle>> {
        self.send_command(Command::R).await?;
        Ok(ElevatorController::<Idle> {
            state: self.state,
            tx: self.tx,
            _marker: PhantomData,
        })
    }
}

#[async_trait]
impl Transition for ElevatorController<Idle> {
    async fn on_event(self: Box<Self>, event: Event) -> Box<dyn Transition + Send + 'static> {
        self
    }
}

pub async fn my_event_handler(event: Event) -> Result<(), ()> {
    println!("Handling event: {:?}", event);
    Ok(())
}
