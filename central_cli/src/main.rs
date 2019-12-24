mod cli;
mod modules;

use modules::Connect;

//use std::error::Error;

//use tokio::prelude::*;

fn main() {
    let matches = cli::get_cli_matches();

    if let Some(runner) = Connect::matches(matches) {
        runner.run()
    }
}
