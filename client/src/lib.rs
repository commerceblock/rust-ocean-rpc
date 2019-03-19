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
extern crate bitcoin_amount;
extern crate bitcoin_hashes;
extern crate hex;
extern crate jsonrpc;
extern crate num_bigint;
extern crate secp256k1;
extern crate serde;
extern crate serde_json;
extern crate rust_ocean;

pub extern crate ocean_rpc_json;
pub use ocean_rpc_json as json;
pub use ocean_rpc_json::getters::*;

mod client;
mod error;
mod queryable;

pub use client::*;
pub use error::Error;
pub use queryable::*;
