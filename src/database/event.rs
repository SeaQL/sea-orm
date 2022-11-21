use crate::DbErr;

pub trait EventStream {
    type Sender: EventSender;
    type Receiver: EventReceiver;

    fn subscribe(self) -> (Self::Sender, Self::Receiver);
}

#[async_trait::async_trait]
pub trait EventSender {
    async fn send(&self, event: Event) -> Result<(), DbErr>;
}

#[async_trait::async_trait]
pub trait EventReceiver {
    async fn recv(&mut self) -> Result<Event, DbErr>;
}

#[derive(Debug, Clone)]
pub enum Event {
    Insert,
    Update,
    Delete,
}

mod event_stream_tokio {
    use super::*;
    use tokio::sync::broadcast::{Receiver, Sender};

    impl EventStream for (Sender<Event>, Receiver<Event>) {
        type Sender = Sender<Event>;
        type Receiver = Receiver<Event>;

        fn subscribe(self) -> (Self::Sender, Self::Receiver) {
            self
        }
    }

    #[async_trait::async_trait]
    impl EventSender for Sender<Event> {
        async fn send(&self, event: Event) -> Result<(), DbErr> {
            self.send(event).map(|_| ()).map_err(|e| todo!())
        }
    }

    #[async_trait::async_trait]
    impl EventReceiver for Receiver<Event> {
        async fn recv(&mut self) -> Result<Event, DbErr> {
            self.recv().await.map_err(|e| todo!())
        }
    }
}

mod event_stream_async_channel {
    use super::*;
    use async_channel::{Receiver, Sender};

    impl EventStream for (Sender<Event>, Receiver<Event>) {
        type Sender = Sender<Event>;
        type Receiver = Receiver<Event>;

        fn subscribe(self) -> (Self::Sender, Self::Receiver) {
            self
        }
    }

    #[async_trait::async_trait]
    impl EventSender for Sender<Event> {
        async fn send(&self, event: Event) -> Result<(), DbErr> {
            self.send(event).await.map_err(|e| todo!())
        }
    }

    #[async_trait::async_trait]
    impl EventReceiver for Receiver<Event> {
        async fn recv(&mut self) -> Result<Event, DbErr> {
            self.recv().await.map_err(|e| todo!())
        }
    }
}
