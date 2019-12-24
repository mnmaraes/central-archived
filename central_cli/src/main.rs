mod cli;

use std::error::Error;

use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::get_cli_matches();

    match matches.subcommand() {
        ("connect", Some(conn_matches)) => {
            if conn_matches.is_present("interactive") {
                run_interactive_connect().await;
                return;
            }

            run_connect(conn_matches.value_of("message").unwrap()).await;
        }
        _ => {
            println!("Nothing matched");
        }
    }
}

async fn run_interactive_connect() -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn run_connect(message: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}
