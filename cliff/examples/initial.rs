use std::collections::HashMap;

use failure::Error;

use futures::prelude::*;

use rand::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut rng = thread_rng();
    let wining_number: u8 = rng.gen();

    // 1. Set up Server
    let server = cliff::create_server()?;

    let num_clients = 10;
    let num_guesses = 5000000;

    // TODO: Spawn server and clients on different threads
    let results = server
        .take(num_guesses)
        .enumerate()
        .filter_map(|(_, message)| {
            async move {
                if let Some(m) = message.as_map() {
                    // TODO: Need to Ser/De this so it's easier to deal with
                    let mut hash: HashMap<String, u8> = HashMap::new();
                    for (key_value, entry) in m {
                        match (key_value.as_str(), entry.as_u64()) {
                            (Some(key), Some(value)) => {
                                hash.entry(key.to_string()).or_insert(value as u8);
                            }
                            _ => {}
                        }
                    }

                    match (hash.get("client_id"), hash.get("guess")) {
                        (Some(id), Some(guess)) => {
                            //println!("{}: client {} guessed {}", i, *id, guess);
                            if *guess == wining_number {
                                //println!("Guess from {} is a winner!", id);
                                Some(id.to_owned())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
        })
        .collect();

    // 2. Spawn Clients
    let clients: Vec<_> = stream::iter(0..num_clients)
        .filter_map(|i: u16| {
            async move {
                match cliff::create_client().await {
                    Ok(c) => Some((i, c)),
                    Err(e) => {
                        println!("error: {}", e);
                        None
                    }
                }
            }
        })
        .collect()
        .await;

    // 3. Pass messages from Clients to Server
    for _ in 0..num_guesses {
        let (id, client) = clients.choose(&mut rng).unwrap();
        let guess: u8 = rng.gen();

        let map = vec![
            (rmpv::Value::from("client_id"), rmpv::Value::from(*id)),
            (rmpv::Value::from("guess"), rmpv::Value::from(guess)),
        ];

        let _ = client.send(rmpv::Value::from(map));
    }
    // 4. Have Server issue Event messages
    //   - Have Clients receive only Messages they are interested in
    let _: Vec<_> = results.await;

    Ok(())
}
