use std::collections::HashSet;

use cliff::{Handler, Message, UnixConnection, UnixServer};

// Cat Shelter
// Data
struct Cat {
    id: usize,
    name: String,
    has_owner: bool,
    aggressiveness: u8,
}

// Client Messages
#[derive(Message)]
struct Subscribe(usize);

#[derive(Message)]
struct Unsubscribe(usize);

#[derive(Message)]
struct Boogey {}
//enum ClientMessage {
//Subscribe,
//Unsubscribe,
//Pet(usize),
//Adopt(usize),
//Abandon(Cat),
//}

// Responses
#[derive(Message)]
struct Synchronize(Vec<Cat>);

#[derive(Message)]
struct Update(Cat);
//enum ServerMessage {
//Synch(Vec<Cat>),
//Update(Cat),
//}

// Server/Clients
#[derive(Default)]
struct Server {
    subscribers: HashSet<usize>,
}

impl Handler<UnixConnection> for Server {
    fn handle(&mut self, message: &mut UnixConnection) {
        // TODO: Implement and store connection's read side
        unimplemented!()
    }
}

impl Handler<Subscribe> for Server {
    fn handle(&mut self, message: &mut Subscribe) {
        self.subscribers.insert(message.0);
    }
}

impl Handler<Unsubscribe> for Server {
    fn handle(&mut self, message: &mut Unsubscribe) {
        self.subscribers.remove(&message.0);
    }
}

#[tokio::main]
async fn main() {
    // Spawn Server
    let runtime = Server::serve();

    // Example Message Sending
    runtime.send(Subscribe(2));
    runtime.send(Unsubscribe(2));

    // TODO: Actually write Server code

    // Spawn Clients
    // tokio::spawn(async {});

    //tokio::signal::ctrl_c().await.ok();
    println!("Shutting down gracefully");
}
