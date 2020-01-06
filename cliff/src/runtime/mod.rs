use std::ops::Deref;

use tokio::sync::mpsc;

use futures::stream::StreamExt;

// Runtime
pub trait Message {}

pub trait Handler<M: Message> {
    fn handle(&mut self, message: &M);
}

pub trait Handled<T>: Message {
    fn be_handled(&self, actor: &mut T);
}

impl<T, M: Message> Handled<T> for M
where
    T: Handler<M>,
{
    fn be_handled(&self, actor: &mut T) {
        actor.handle(self);
    }
}

pub struct Handle<T>(mpsc::UnboundedReceiver<Box<dyn Handled<T> + Send>>);

pub struct Address<T>(mpsc::UnboundedSender<Box<dyn Handled<T> + Send>>);
impl<T> Address<T> {
    pub fn send<M: Handled<T> + Send + Sync + 'static>(&self, message: M) {
        self.0.send(Box::new(message)).ok();
    }
}

pub struct Runtime<T> {
    addr: Address<T>,
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
        loop {
            let message = handle.0.next().await.unwrap();
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
