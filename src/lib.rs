#![cfg_attr(feature="cargo-clippy", allow(zero_prefixed_literal))]

#[macro_use]
extern crate exonum;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

extern crate rand;
extern crate rug;
extern crate vdf;

pub mod api;
pub mod blockchain;
pub mod rng;
mod service;

pub use service::{ExonumRngService, SERVICE_NAME, SERVICE_ID};