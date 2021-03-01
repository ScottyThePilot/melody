#[macro_use] extern crate serde;
#[macro_use] extern crate util_macros;
extern crate serenity;
extern crate serde_multi;
extern crate tokio;

pub mod commands;
pub mod data;
pub mod error;
pub mod handler;

#[tokio::main]
async fn main() {}
