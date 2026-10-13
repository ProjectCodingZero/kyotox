//! Entry point for the Herrumbre application executable.
//!
//! Initializes logging, panic hooks, thread pools, terminal state via Ratatui,
//! and runs the main async event loop.
//!

mod my_clap;

use herrumbre_core::logger;

//Extern uses
use clap::{Command, Parser};
use color_eyre::eyre::{Result, eyre};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
fn main() {
    let args = my_clap::Command::parse();
    _ = logger::setup_logger(args.log);
    println!("{:?}", args);
}
