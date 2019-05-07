// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! A very simple example used as a self-test of this library against an
//! Ocean node.
extern crate bitcoin;
extern crate ocean_rpc;
extern crate rust_ocean;

use ocean_rpc::{Client, Error, RpcApi};

fn main_result() -> Result<(), Error> {
    let mut args = std::env::args();

    let _exe_name = args.next().unwrap();

    let url = args.next().expect("Usage: <rpc_url> [username] [password]");
    let user = args.next();
    let pass = args.next();

    let rpc = Client::new(url, user, pass);

    let blockchain_info = rpc.get_blockchain_info()?;
    println!("info\n{:?}", blockchain_info);

    let best_block_hash = rpc.get_best_block_hash()?;
    println!("best block hash: {}", best_block_hash);
    let ocean_block: rust_ocean::Block = rpc.get_by_id(&best_block_hash)?;
    println!("block\n{:?}", ocean_block);
    let ocean_tx: rust_ocean::Transaction = rpc.get_by_id(&ocean_block.txdata[0].txid())?;
    println!("tx\n{:?}", ocean_tx);

    Ok(())
}

fn main() {
    match main_result() {
        Err(e) => println!("{}", e),
        _ => (),
    };
}
