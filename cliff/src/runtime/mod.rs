use std::ops::Deref;

use serde::{Deserialize, Serialize};

use tokio::sync::mpsc;

// Runtime
pub trait Message: Send + Sync {}

pub trait Handler<M: Message> {
    fn handle(&mut self, message: &mut M);
}

pub trait Handled<T>: Message {
    fn be_handled(&mut self, actor: &mut T);
    fn be_forwaded(self: Box<Self>, runtime: &Runtime<T>);
}

impl<T: Default + Send + 'static, M: Message + 'static> Handled<T> for M
where
    T: Handler<M>,
{
    fn be_handled(&mut self, actor: &mut T) {
        actor.handle(self);
    }

    fn be_forwaded(self: Box<Self>, runtime: &Runtime<T>) {
        runtime.forward(self)
    }
}

pub struct Handle<T>(mpsc::UnboundedReceiver<Box<dyn Handled<T> + Send>>);

pub struct Address<T>(mpsc::UnboundedSender<Box<dyn Handled<T> + Send>>);
impl<T> Address<T> {
    pub fn send<M: Handled<T> + Send + Sync + 'static>(&self, message: M) {
        // TODO: Better Error Handling
        self.0.send(Box::new(message)).ok();
    }

    pub fn forward<M: Handled<T> + Send + Sync + 'static>(&self, message: Box<M>) {
        self.0.send(message).ok();
    }
}

pub struct Runtime<T> {
    addr: Address<T>,
}

impl<T> Clone for Runtime<T> {
    fn clone(&self) -> Runtime<T> {
        let subject = self.addr.0.clone();

        Runtime::<T> {
            addr: Address(subject),
        }
    }
}

impl<T: Default + Send + 'static> Deref for Runtime<T> {
    type Target = Address<T>;

    fn deref(&self) -> &Address<T> {
        &self.addr
    }
}

impl<T: Default + Send + 'static> Runtime<T> {
    fn run() -> Self {
        let (subject, stream) = mpsc::unbounded_channel::<Box<dyn Handled<T> + Send>>();
        let handle = Handle(stream);

        dispatch(handle);

        Self {
            addr: Address(subject),
        }
    }
}

fn dispatch<T: Default + Send + 'static>(mut handle: Handle<T>) {
    tokio::spawn(async move {
        let mut dispatched = T::default();
        while let Some(mut message) = handle.0.recv().await {
            message.be_handled(&mut dispatched);
        }
    });
}

pub trait SelfStarter: Sized {
    fn start() -> Runtime<Self>;
}

impl<T: Default + Send + 'static> SelfStarter for T {
    fn start() -> Runtime<T> {
        Runtime::run()
    }
}
