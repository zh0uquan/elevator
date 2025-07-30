use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::cmd::Command;
use crate::types::sched_events::Action;

pub type SharedStateMachine = Arc<Mutex<Option<BoxedTransition>>>;

pub type BoxedTransition = Box<dyn Transition + Sync + Send + 'static>;
#[async_trait]
pub trait Transition: Send + 'static + Sync + Debug {
    async fn on_event(
        self: Box<Self>,
        action: Action,
    ) -> anyhow::Result<Box<dyn Transition + Sync + Send + 'static>>;
}

pub trait IntoBoxedTransition {
    fn boxed(self) -> BoxedTransition;
}

impl<T> IntoBoxedTransition for T
where
    T: Transition + Send + Sync + 'static,
{
    fn boxed(self) -> BoxedTransition {
        Box::new(self)
    }
}

#[derive(Debug)]
pub struct ElevatorState<State> {
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
    _marker: PhantomData<State>,
}

impl<State> ElevatorState<State> {
    async fn send_command(&self, command: Command) -> anyhow::Result<()> {
        self.tx.send(command)?;
        Ok(())
    }

    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<Command>) -> ElevatorState<State> {
        ElevatorState::<State> {
            tx,
            _marker: PhantomData,
        }
    }

    pub fn transit<NextState>(self) -> ElevatorState<NextState> {
        ElevatorState::<NextState> {
            tx: self.tx,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct PreStart;
#[derive(Debug)]
pub struct Idle;
#[derive(Debug)]
pub struct MovingUp;
#[derive(Debug)]
pub struct MovingDown;
#[derive(Debug)]
pub struct DoorOpening;
#[derive(Debug)]
pub struct DoorOpened;
#[derive(Debug)]
pub struct DoorClosing;
#[derive(Debug)]
pub struct DoorClosed;
#[derive(Debug)]
pub struct Braking;
#[derive(Debug)]
pub struct EmergencyBrake;

impl ElevatorState<PreStart> {
    pub async fn init(self) -> anyhow::Result<ElevatorState<Idle>> {
        self.send_command(Command::R).await?;
        Ok(self.transit::<Idle>())
    }
}

#[async_trait]
impl Transition for ElevatorState<Idle> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::MovingUp => {
                println!("Moving up");
                self.send_command(Command::MU).await?;
                Ok(self.transit::<MovingUp>().boxed())
            }
            Action::MovingDown => {
                println!("Moving down");
                self.send_command(Command::MU).await?;
                Ok(self.transit::<MovingDown>().boxed())
            }
            Action::OpeningDoor => {
                println!("Opening door");
                self.send_command(Command::DO).await?;
                Ok(self.transit::<DoorOpening>().boxed())
            }
            Action::Braking => {
                eprintln!("Can't Brake, already Stopped.");
                Ok(self)
            }
            Action::Stopped => {
                eprintln!("Already Stopped.");
                Ok(self)
            }
            Action::DoorClosed => {
                eprintln!("Door Already Closed");
                Ok(self)
            }
            Action::DoorOpened | Action::ClosingDoor => {
                eprintln!(
                    "Strange door status: {:?}, state in {:?}",
                    action, self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<MovingUp> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::Braking => {
                println!("Braking.");
                self.send_command(Command::S).await?;
                Ok(self.transit::<Braking>().boxed())
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<MovingDown> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::Braking => {
                println!("Braking.");
                self.send_command(Command::S).await?;
                Ok(self.transit::<Braking>().boxed())
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<Braking> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::Stopped => {
                println!("Stopped.");
                Ok(self.transit::<Idle>().boxed())
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<DoorOpening> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::DoorOpened => {
                println!("Door Opened.");
                Ok(self.transit::<DoorOpened>().boxed())
            }
            Action::OpeningDoor => {
                println!("Double Opening Door.");
                Ok(self)
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<DoorOpened> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::ClosingDoor => {
                println!("Closing Door.");
                self.send_command(Command::DC).await?;
                Ok(self.transit::<DoorClosing>().boxed())
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<DoorClosing> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::DoorClosed => {
                println!("Door Closed.");
                Ok(self.transit::<Idle>().boxed())
            }
            Action::ClosingDoor => {
                println!("Double Closing Door.");
                Ok(self)
            }
            ev => {
                eprintln!(
                    "Ignored: invalid schedule event {ev:?} in state {:?}",
                    self._marker
                );
                Ok(self)
            }
        }
    }
}
