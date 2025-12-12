mod app;
mod cli;
mod cmds;
mod config;
mod domain;
mod error;
mod logging;
mod repository;
mod service;
mod utils;
mod view;

#[tokio::main]
async fn main() {
    if let Err(e) = app::run().await {
        let follow_up = e.follow_up();
        let is_unexpected = e.is_unexpected();

        eprintln!("Error: {:?}", anyhow::anyhow!(e));

        if let Some(f) = follow_up {
            eprintln!(
                "
{f}"
            );
        }

        if is_unexpected {
            eprint!(
                "
---
This error is unexpected. 
Let @dhth know about this via https://github.com/dhth/grafq/issues.
"
            );
        }

        std::process::exit(1);
    }
}
