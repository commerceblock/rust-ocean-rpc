// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Rust Client for Ocean API
//!
//! This is a client library for the Ocean JSON-RPC API.
//!

#![crate_name = "ocean_rpc"]
#![crate_type = "rlib"]

#[macro_use]
extern crate log;
extern crate bitcoin;
extern crate hex;
extern crate jsonrpc;
extern crate num_bigint;
extern crate rust_ocean;
#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

pub extern crate ocean_rpc_json;
pub use ocean_rpc_json as json;

mod client;
mod error;
mod queryable;

pub use client::*;
pub use error::Error;
pub use queryable::*;
