use crate::subscriber::{
    Subscriber,
    SubscriberCallback,
    SubscriberConfig,
    SubscriberError,
    SubscriberEvent,
    SubscriberEventType,
};
use anyhow::anyhow;
use std::{
    fmt::{Debug, Display, Formatter},
    panic::{catch_unwind, RefUnwindSafe, UnwindSafe},
};

#[derive(Clone, Debug)]
pub enum SEvent {
    Event1(String),
    Event2(String),
    Kill,
}

impl UnwindSafe for SEvent {}

impl<F: Sized + Sync, SEvent> SubscriberCallback<SEvent> for F
    where
        Self: Fn(SEvent) + Send + UnwindSafe + RefUnwindSafe,
        SEvent: SubscriberEvent,
{}

impl SubscriberEvent for SEvent {
    type Type = SEventType;

    fn should_kill(&self) -> bool {
        matches!(self, SEvent::Kill)
    }

    fn get_type(&self) -> Self::Type {
        match self {
            SEvent::Event1(_) => SEventType::EventType1,
            SEvent::Event2(_) => SEventType::EventType2,
            SEvent::Kill => SEventType::EventTypeKill,
        }
    }
}


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
#[repr(C)]
pub enum SEventType {
    /// Event 1 maps to 0x0
    EventType1,
    /// Event 2 maps to 0x1
    EventType2,
    /// Kill map to 0x2
    EventTypeKill,
}

impl Display for SEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SubscriberEventType for SEventType {}

impl SEventType {
    pub const EVENT_1: u16 = 0x0;
    pub const EVENT_2: u16 = 0x1;
    pub const EVENT_KILL: u16 = 0x2;
}

#[derive(Debug)]
pub struct SConfig {
    channel_size: u32,
    sub_count: u32,
}

impl SubscriberConfig for SConfig {
    fn subscriber_count(&self) -> usize {
        self.sub_count as usize
    }

    fn channel_size(&self) -> usize {
        self.channel_size as usize
    }
}

impl Default for SConfig {
    fn default() -> Self {
        Self {
            channel_size: 1024,
            sub_count: 1,
        }
    }
}

impl TryFrom<u16> for SEventType {
    type Error = anyhow::Error;

    fn try_from(n: u16) -> Result<Self, Self::Error> {
        Ok(match n {
            SEventType::EVENT_1 => Self::EventType1,
            SEventType::EVENT_2 => Self::EventType2,
            SEventType::EVENT_KILL => Self::EventTypeKill,
            _ => {
                return Err(anyhow!("Not a valid Event type"));
            }
        })
    }
}

impl From<SEventType> for u16 {
    fn from(event_type: SEventType) -> Self {
        match event_type {
            SEventType::EventType1 => SEventType::EVENT_1,
            SEventType::EventType2 => SEventType::EVENT_2,
            SEventType::EventTypeKill => SEventType::EVENT_KILL,
        }
    }
}

pub struct Sub<Event> {
    callback: Box<dyn SubscriberCallback<Event>>,
}

impl Sub<SEvent> {
    pub fn new<T: SubscriberCallback<SEvent> + 'static>(callback: T) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl Subscriber<SEvent, SError> for Sub<SEvent> {
    fn new<T: SubscriberCallback<SEvent> + 'static>(callback: T) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }

    fn notify(&self, event: SEvent) -> Result<(), SError> {
        match catch_unwind(move || (self.callback)(event)) {
            Ok(..) => Ok(()),
            Err(e) => Err(anyhow!(format!("Error while calling callback: {:?}", e)).into()),
        }
    }
}

#[derive(Debug)]
pub struct SError {
    error: anyhow::Error,
}

impl SubscriberError for SError {}

impl From<anyhow::Error> for SError {
    fn from(error: anyhow::Error) -> Self {
        Self { error }
    }
}

impl Display for SError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for SError {}
