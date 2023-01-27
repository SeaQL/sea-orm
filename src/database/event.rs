use crate::{DbErr, EntityTrait};
use async_trait::async_trait;
use sea_query::{DynIden, Value};
use std::{any::TypeId, collections::HashMap, fmt::Debug};

pub trait EventStream {
    type Sender: EventSender;
    type Receiver: EventReceiver;

    fn subscribe(self) -> (Self::Sender, Self::Receiver);
}

#[async_trait]
pub trait EventSender {
    async fn send(&self, event: Event) -> Result<(), DbErr>;
}

#[async_trait]
pub trait EventReceiver {
    async fn recv(&mut self) -> Result<Event, DbErr>;
}

#[derive(Debug, Clone)]
pub struct Event {
    pub entity_type_id: TypeId,
    pub action: EventAction,
    pub values: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum EventAction {
    Insert,
    Update,
    Delete,
}

impl Event {
    pub fn of_entity<E>(&self) -> bool
    where
        E: EntityTrait,
    {
        self.entity_type_id == TypeId::of::<E>()
    }
}

mod impl_event_stream_for_async_broadcast {
    use super::*;
    use async_broadcast::{Receiver, Sender};
    use futures::FutureExt;

    impl EventStream for (Sender<Event>, Receiver<Event>) {
        type Sender = Sender<Event>;
        type Receiver = Receiver<Event>;

        fn subscribe(self) -> (Self::Sender, Self::Receiver) {
            self
        }
    }

    #[async_trait]
    impl EventSender for Sender<Event> {
        async fn send(&self, event: Event) -> Result<(), DbErr> {
            self.broadcast(event).await.map(|_| ()).map_err(|e| todo!())
        }
    }

    #[async_trait]
    impl EventReceiver for Receiver<Event> {
        async fn recv(&mut self) -> Result<Event, DbErr> {
            self.recv().await.map_err(|e| todo!())
        }
    }
}
