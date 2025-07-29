use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::strategies::scan::SchedulerState;
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
    state: Arc<Mutex<SchedulerState>>,
    tx: tokio::sync::mpsc::UnboundedSender<Command>,
    _marker: PhantomData<State>,
}

impl<State> ElevatorState<State> {
    async fn send_command(&self, command: Command) -> anyhow::Result<()> {
        self.tx.send(command)?;
        Ok(())
    }

    pub fn new(
        state: Arc<Mutex<SchedulerState>>,
        tx: tokio::sync::mpsc::UnboundedSender<Command>,
    ) -> ElevatorState<State> {
        ElevatorState::<State> {
            state,
            tx,
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
        Ok(ElevatorState::<Idle>::new(Arc::clone(&self.state), self.tx))
    }
}

impl ElevatorState<Idle> {
    async fn moving_up(self: Box<Self>) -> anyhow::Result<BoxedTransition> {
        self.send_command(Command::MU).await?;
        Ok(Box::new(ElevatorState::<MovingUp>::new(
            Arc::clone(&self.state),
            self.tx,
        )))
    }

    async fn moving_down(self: Box<Self>) -> anyhow::Result<BoxedTransition> {
        self.send_command(Command::MD).await?;
        Ok(Box::new(ElevatorState::<MovingDown>::new(
            Arc::clone(&self.state),
            self.tx,
        )))
    }

    async fn opening_door(self: Box<Self>) -> anyhow::Result<BoxedTransition> {
        self.send_command(Command::DO).await?;
        Ok(Box::new(ElevatorState::<DoorOpening>::new(
            Arc::clone(&self.state),
            self.tx,
        )))
    }
}

#[async_trait]
impl Transition for ElevatorState<Idle> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        let direction_up = {
            let state = self.state.lock().await;
            state.direction_up
        };
        match action {
            Action::Moving(v) => {
                if direction_up {
                    println!("Moving up to floor {v}");
                    self.moving_up().await
                } else {
                    println!("Moving down to floor {v}");
                    self.moving_down().await
                }
            }
            Action::OpeningDoor => {
                println!("Opening door");
                self.opening_door().await
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

impl ElevatorState<MovingUp> {
    async fn brake(self: Box<Self>) -> anyhow::Result<BoxedTransition> {
        self.send_command(Command::S).await?;
        Ok(Box::new(ElevatorState::<Braking>::new(
            Arc::clone(&self.state),
            self.tx,
        )))
    }
}

impl ElevatorState<MovingDown> {
    async fn brake(self: Box<Self>) -> anyhow::Result<BoxedTransition> {
        self.send_command(Command::S).await?;
        Ok(Box::new(ElevatorState::<Braking>::new(
            Arc::clone(&self.state),
            self.tx,
        )))
    }
}

#[async_trait]
impl Transition for ElevatorState<MovingUp> {
    async fn on_event(self: Box<Self>, action: Action) -> anyhow::Result<BoxedTransition> {
        match action {
            Action::Braking => {
                println!("Braking.");
                self.brake().await
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
                self.brake().await
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
                Ok(Box::new(ElevatorState::<Idle>::new(
                    Arc::clone(&self.state),
                    self.tx,
                )))
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
                Ok(Box::new(ElevatorState::<DoorOpened>::new(
                    Arc::clone(&self.state),
                    self.tx,
                )))
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
                Ok(Box::new(ElevatorState::<DoorClosing>::new(
                    Arc::clone(&self.state),
                    self.tx,
                )))
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
                Ok(Box::new(ElevatorState::<Idle>::new(
                    Arc::clone(&self.state),
                    self.tx,
                )))
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
