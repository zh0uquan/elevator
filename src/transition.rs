use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::strategy::SchedulerState;
use crate::{Command, ScheduleEvent};

pub type BoxedTransition = Box<dyn Transition + Sync + Send + 'static>;
#[async_trait]
pub trait Transition: Send + 'static + Sync + Debug {
    async fn on_event(
        self: Box<Self>,
        event: ScheduleEvent,
    ) -> anyhow::Result<Box<dyn Transition + Sync + Send + 'static>>;
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
pub struct DoorOpen;
#[derive(Debug)]
pub struct DoorClosing;
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
    async fn on_event(self: Box<Self>, event: ScheduleEvent) -> anyhow::Result<BoxedTransition> {
        let direction_up = {
            let state = self.state.lock().await;
            state.direction_up
        };
        match event {
            ScheduleEvent::Moving(v) => {
                if direction_up {
                    println!("Moving up to floor {v}");
                    self.moving_up().await
                } else {
                    println!("Moving down to floor {v}");
                    self.moving_down().await
                }
            }
            ScheduleEvent::OpeningDoor => self.opening_door().await,
            ScheduleEvent::Braking => {
                eprintln!("Can't Brake, already Stopped.");
                Ok(self)
            }
            ScheduleEvent::Stopped => {
                eprintln!("Already Stopped.");
                Ok(self)
            }
            ScheduleEvent::DoorOpened => Ok(self),
            ScheduleEvent::ClosingDoor => Ok(self),
            ScheduleEvent::DoorClosed => Ok(self),
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
    async fn on_event(self: Box<Self>, event: ScheduleEvent) -> anyhow::Result<BoxedTransition> {
        match event {
            ScheduleEvent::Braking => {
                println!("Braking.");
                self.brake().await
            }
            ev => {
                eprintln!("Ignored: invalid schedule event {ev:?}");
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<MovingDown> {
    async fn on_event(self: Box<Self>, event: ScheduleEvent) -> anyhow::Result<BoxedTransition> {
        Ok(self)
    }
}

#[async_trait]
impl Transition for ElevatorState<Braking> {
    async fn on_event(self: Box<Self>, event: ScheduleEvent) -> anyhow::Result<BoxedTransition> {
        match event {
            ScheduleEvent::Stopped => {
                println!("Stopped.");
                Ok(Box::new(ElevatorState::<Idle>::new(
                    Arc::clone(&self.state),
                    self.tx,
                )))
            }
            ev => {
                eprintln!("Ignored: invalid schedule event {ev:?}");
                Ok(self)
            }
        }
    }
}

#[async_trait]
impl Transition for ElevatorState<DoorOpening> {
    async fn on_event(self: Box<Self>, event: ScheduleEvent) -> anyhow::Result<BoxedTransition> {
        Ok(self)
    }
}
