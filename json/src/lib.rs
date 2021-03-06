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

#![crate_name = "ocean_rpc_json"]
#![crate_type = "rlib"]

extern crate bitcoin;
extern crate hex;
extern crate num_bigint;
extern crate rust_ocean;
#[allow(unused)]
#[macro_use] // `macro_use` is needed for v1.24.0 compilation.
extern crate serde;
extern crate serde_json;

use std::str::FromStr;

use bitcoin::hashes::{sha256, sha256d};
use bitcoin::util::bip158;
use bitcoin::{Amount, PrivateKey, PublicKey, Script};
use num_bigint::BigUint;
use rust_ocean::{encode, Address, Transaction};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

//TODO(stevenroose) consider using a Time type

/// A module used for serde serialization of bytes in hexadecimal format.
///
/// The module is compatible with the serde attribute.
pub mod serde_hex {
    use hex;
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(&b))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let hex_str: String = ::serde::Deserialize::deserialize(d)?;
        Ok(hex::decode(hex_str).map_err(D::Error::custom)?)
    }

    pub mod opt {
        use hex;
        use serde::de::Error;
        use serde::{Deserializer, Serializer};

        pub fn serialize<S: Serializer>(b: &Option<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
            match *b {
                None => s.serialize_none(),
                Some(ref b) => s.serialize_str(&hex::encode(&b)),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
            let hex_str: String = ::serde::Deserialize::deserialize(d)?;
            Ok(Some(hex::decode(hex_str).map_err(D::Error::custom)?))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMultiSigAddressResult {
    pub address: Address,
    pub redeem_script: Script,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct LoadWalletResult {
    pub name: String,
    pub warning: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockResult {
    pub hash: sha256d::Hash,
    pub confirmations: u32,
    pub size: usize,
    pub strippedsize: Option<usize>,
    pub weight: usize,
    pub height: usize,
    pub version: u32,
    #[serde(default, with = "::serde_hex::opt")]
    pub version_hex: Option<Vec<u8>>,
    pub merkleroot: sha256d::Hash,
    pub tx: Vec<sha256d::Hash>,
    pub time: usize,
    pub mediantime: Option<usize>,
    pub nonce: u32,
    pub bits: String,
    #[serde(deserialize_with = "deserialize_difficulty")]
    pub difficulty: BigUint,
    #[serde(with = "::serde_hex")]
    pub chainwork: Vec<u8>,
    pub contracthash: sha256d::Hash,
    pub attestationhash: sha256d::Hash,
    pub mappinghash: sha256d::Hash,
    pub previousblockhash: Option<sha256d::Hash>,
    pub nextblockhash: Option<sha256d::Hash>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockHeaderResult {
    pub hash: sha256d::Hash,
    pub confirmations: u32,
    pub height: usize,
    pub version: u32,
    #[serde(default, with = "::serde_hex::opt")]
    pub version_hex: Option<Vec<u8>>,
    pub merkleroot: sha256d::Hash,
    pub time: usize,
    pub mediantime: Option<usize>,
    pub nonce: u32,
    pub bits: String,
    #[serde(deserialize_with = "deserialize_difficulty")]
    pub difficulty: BigUint,
    #[serde(with = "::serde_hex")]
    pub chainwork: Vec<u8>,
    pub contracthash: sha256d::Hash,
    pub attestationhash: sha256d::Hash,
    pub mappinghash: sha256d::Hash,
    pub previousblockhash: Option<sha256d::Hash>,
    pub nextblockhash: Option<sha256d::Hash>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMiningInfoResult {
    pub blocks: u32,
    pub currentblockweight: Option<u64>,
    pub currentblocktx: Option<usize>,
    #[serde(deserialize_with = "deserialize_difficulty")]
    pub difficulty: BigUint,
    pub networkhashps: f64,
    pub pooledtx: usize,
    pub chain: String,
    pub warnings: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResultVinScriptSig {
    pub asm: String,
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
}

impl GetRawTransactionResultVinScriptSig {
    pub fn script(&self) -> Result<Script, encode::Error> {
        Ok(Script::from(self.hex.clone()))
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResultVin {
    pub sequence: u32,
    /// The raw scriptSig in case of a coinbase tx.
    #[serde(default, with = "::serde_hex::opt")]
    pub coinbase: Option<Vec<u8>>,
    /// Not provided for coinbase txs.
    pub txid: Option<sha256d::Hash>,
    /// Not provided for coinbase txs.
    pub vout: Option<u32>,
    /// The scriptSig in case of a non-coinbase tx.
    pub script_sig: Option<GetRawTransactionResultVinScriptSig>,
    /// Not provided for coinbase txs.
    #[serde(default, deserialize_with = "deserialize_hex_array_opt")]
    pub txinwitness: Option<Vec<Vec<u8>>>,
    /// Pegin flag
    #[serde(rename = "is_pegin")]
    pub is_pegin: Option<bool>,
}

impl GetRawTransactionResultVin {
    /// Whether this input is from a coinbase tx.
    /// The [txid], [vout] and [script_sig] fields are not provided
    /// for coinbase transactions.
    pub fn is_coinbase(&self) -> bool {
        self.coinbase.is_some()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResultVoutScriptPubKey {
    pub asm: String,
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
    pub req_sigs: Option<usize>,
    #[serde(rename = "type")]
    pub type_: Option<String>, //TODO(stevenroose) consider enum
    pub addresses: Option<Vec<Address>>,
}

impl GetRawTransactionResultVoutScriptPubKey {
    pub fn script(&self) -> Result<Script, encode::Error> {
        Ok(Script::from(self.hex.clone()))
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResultVout {
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub value: Amount,
    pub n: u32,
    pub script_pub_key: GetRawTransactionResultVoutScriptPubKey,
    pub asset: sha256d::Hash,
    pub assetlabel: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRawTransactionResult {
    #[serde(rename = "in_active_chain")]
    pub in_active_chain: Option<bool>,
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
    pub txid: sha256d::Hash,
    pub hash: sha256d::Hash,
    pub size: usize,
    pub vsize: usize,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<GetRawTransactionResultVin>,
    pub vout: Vec<GetRawTransactionResultVout>,
    pub blockhash: Option<sha256d::Hash>,
    pub confirmations: Option<u32>,
    pub time: Option<usize>,
    pub blocktime: Option<usize>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct GetBlockFilterResult {
    pub header: sha256::Hash,
    #[serde(with = "::serde_hex")]
    pub filter: Vec<u8>,
}

impl GetBlockFilterResult {
    /// Get the filter.
    /// Note that this copies the underlying filter data. To prevent this,
    /// use [into_filter] instead.
    pub fn to_filter(&self) -> bip158::BlockFilter {
        bip158::BlockFilter::new(&self.filter)
    }

    /// Convert the result in the filter type.
    pub fn into_filter(self) -> bip158::BlockFilter {
        bip158::BlockFilter {
            content: self.filter,
        }
    }
}

impl GetRawTransactionResult {
    /// Whether this tx is a coinbase tx.
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].is_coinbase()
    }

    pub fn transaction(&self) -> Result<Transaction, encode::Error> {
        Ok(encode::deserialize(&self.hex)?)
    }
}

/// Enum to represent the BIP125 replaceable status for a transaction.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Bip125Replaceable {
    Yes,
    No,
    Unknown,
}

/// Enum to represent to sendanytoaddress rpc result
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SendAnyToAddressResult {
    Txid(sha256d::Hash),
    Txids(Vec<sha256d::Hash>),
}

/// Enum to represent the BIP125 replaceable status for a transaction.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GetTransactionResultDetailCategory {
    Send,
    Receive,
    Generate,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct GetTransactionResultDetail {
    pub address: Address,
    pub category: GetTransactionResultDetailCategory,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub amount: Amount,
    pub label: Option<String>,
    pub vout: u32,
    #[serde(default, with = "bitcoin::util::amount::serde::as_btc::opt")]
    pub fee: Option<Amount>,
    pub abandoned: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct WalletTxInfo {
    pub confirmations: i32,
    pub blockhash: Option<sha256d::Hash>,
    pub blockindex: Option<usize>,
    pub blocktime: Option<u64>,
    pub txid: sha256d::Hash,
    pub time: u64,
    pub timereceived: u64,
    #[serde(rename = "bip125-replaceable")]
    pub bip125_replaceable: Bip125Replaceable,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct GetTransactionResult {
    #[serde(flatten)]
    pub info: WalletTxInfo,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub amount: Amount,
    #[serde(default, with = "bitcoin::util::amount::serde::as_btc::opt")]
    pub fee: Option<Amount>,
    pub details: Vec<GetTransactionResultDetail>,
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
}

impl GetTransactionResult {
    pub fn transaction(&self) -> Result<Transaction, encode::Error> {
        Ok(encode::deserialize(&self.hex)?)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct ListTransactionResult {
    #[serde(flatten)]
    pub info: WalletTxInfo,
    #[serde(flatten)]
    pub detail: GetTransactionResultDetail,

    pub trusted: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTxOutResult {
    pub bestblock: sha256d::Hash,
    pub confirmations: u32,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub value: Amount,
    pub script_pub_key: GetRawTransactionResultVoutScriptPubKey,
    pub coinbase: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUnspentQueryOptions {
    #[serde(default, with = "bitcoin::util::amount::serde::as_btc::opt")]
    pub minimum_amount: Option<Amount>,
    #[serde(default, with = "bitcoin::util::amount::serde::as_btc::opt")]
    pub maximum_amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_count: Option<usize>,
    #[serde(default, with = "bitcoin::util::amount::serde::as_btc::opt")]
    pub maximum_sum_amount: Option<Amount>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUnspentResultEntry {
    pub txid: sha256d::Hash,
    pub vout: u32,
    pub address: Address, // TODO: Address for Ocean
    pub account: Option<String>,
    pub script_pub_key: Script,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub amount: Amount,
    pub amountcommitment: Option<sha256d::Hash>,
    pub asset: sha256d::Hash,
    pub assetcommitment: Option<sha256d::Hash>,
    pub assetlabel: Option<String>,
    pub confirmations: usize,
    pub redeem_script: Option<Script>,
    pub spendable: bool,
    pub solvable: bool,
    pub blinder: sha256d::Hash,
    pub assetblinder: sha256d::Hash,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListReceivedByAddressResult {
    #[serde(rename = "involvesWatchonly")]
    pub involved_watch_only: bool,
    pub address: Address,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub amount: Amount,
    pub confirmations: u32,
    pub label: String,
    pub txids: Vec<sha256d::Hash>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRawTransactionResultError {
    pub txid: sha256d::Hash,
    pub vout: u32,
    pub script_sig: Script,
    pub sequence: u32,
    pub error: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRawTransactionResult {
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
    pub complete: bool,
    pub errors: Option<Vec<SignRawTransactionResultError>>,
}

impl SignRawTransactionResult {
    pub fn transaction(&self) -> Result<Transaction, encode::Error> {
        Ok(encode::deserialize(&self.hex)?)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct TestMempoolAccept {
    pub txid: String,
    pub allowed: bool,
    #[serde(rename = "reject-reason")]
    pub reject_reason: Option<String>,
}

/// Status of a softfork
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Softfork {
    /// Name of softfork
    pub id: String,
    /// Block version
    pub version: u64,
    /// Progress toward rejecting pre-softfork blocks
    pub reject: RejectStatus,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestsResult {
    pub genesis_block: sha256d::Hash,
    pub start_block_height: u32,
    pub end_block_height: u32,
    pub confirmed_block_height: u32,
    pub num_tickets: u32,
    pub decay_const: u32,
    pub fee_percentage: u32,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub start_price: Amount,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub auction_price: Amount,
    pub txid: sha256d::Hash,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestBidsResultBid {
    pub txid: sha256d::Hash,
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub fee_pub_key: PublicKey,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestBidsResult {
    pub genesis_block: sha256d::Hash,
    pub start_block_height: u32,
    pub end_block_height: u32,
    pub confirmed_block_height: u32,
    pub num_tickets: u32,
    pub decay_const: u32,
    pub fee_percentage: u32,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub start_price: Amount,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub auction_price: Amount,
    pub txid: sha256d::Hash,
    pub bids: Vec<GetRequestBidsResultBid>,
}

/// Models the result of "getblockchaininfo"
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlockchainInfoResult {
    // TODO: Use Network from rust-bitcoin
    /// Current network name as defined in BIP70 (main, test, regtest)
    pub chain: String,
    /// The current number of blocks processed in the server
    pub blocks: u64,
    /// The current number of headers we have validated
    pub headers: u64,
    /// The hash of the currently best block
    pub bestblockhash: sha256d::Hash,
    /// The current difficulty
    pub difficulty: f64,
    /// Median time for the current best block
    pub mediantime: u64,
    /// Estimate of verification progress [0..1]
    pub verificationprogress: f64,
    /// Total amount of work in active chain, in hexadecimal
    #[serde(with = "::serde_hex")]
    pub chainwork: Vec<u8>,
    /// If the blocks are subject to pruning
    pub pruned: bool,
    /// Lowest-height complete block stored (only present if pruning is enabled)
    pub pruneheight: Option<u64>,
    /// Whether automatic pruning is enabled (only present if pruning is enabled)
    pub automatic_pruning: Option<bool>,
    /// The target size used by pruning (only present if automatic pruning is enabled)
    pub prune_target_size: Option<u64>,
    // TODO: add a type?
    /// Status of BIP9 softforks in progress
    pub bip9_softforks: Value,
}

/// Models the result of "getsidechaininfo"
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSidechainInfoResult {
    /// The fedpegscript in hex
    pub fedpegscript: String,
    /// The pegged asset type in hex
    pub pegged_asset: String,
    /// The minimum difficulty parent chain header target.
    pub min_peg_diff: String,
    /// The parent genesis blockhash as source of pegged-in funds
    pub parent_blockhash: String,
    /// Object containing PUBKEY_ADDRESS, BLINDED_ADDRESS, SECRET_KEY
    /// and SCRIPT_ADDRESS address type prefixes
    pub addr_prefixes: Value,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ImportMultiRequestScriptPubkey<'a> {
    Address(&'a Address),
    Script(&'a Script),
}

impl<'a> serde::Serialize for ImportMultiRequestScriptPubkey<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            ImportMultiRequestScriptPubkey::Address(ref addr) => {
                #[derive(Serialize)]
                struct Tmp<'a> {
                    pub address: &'a Address,
                };
                serde::Serialize::serialize(
                    &Tmp {
                        address: addr,
                    },
                    serializer,
                )
            }
            ImportMultiRequestScriptPubkey::Script(script) => {
                serializer.serialize_str(&hex::encode(script.as_bytes()))
            }
        }
    }
}

/// A import request for importmulti.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize)]
pub struct ImportMultiRequest<'a> {
    pub timestamp: u64,
    /// If using descriptor, do not also provide address/scriptPubKey, scripts, or pubkeys.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<&'a str>,
    #[serde(rename = "scriptPubKey", skip_serializing_if = "Option::is_none")]
    pub script_pubkey: Option<ImportMultiRequestScriptPubkey<'a>>,
    #[serde(rename = "redeemscript", skip_serializing_if = "Option::is_none")]
    pub redeem_script: Option<&'a Script>,
    #[serde(rename = "witnessscript", skip_serializing_if = "Option::is_none")]
    pub witness_script: Option<&'a Script>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub pubkeys: &'a [PublicKey],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub keys: &'a [PrivateKey],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(usize, usize)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchonly: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keypool: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize)]
pub struct ImportMultiOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rescan: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct ImportMultiResultError {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct ImportMultiResult {
    pub success: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub error: Option<ImportMultiResultError>,
}

/// Progress toward rejecting pre-softfork blocks
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct RejectStatus {
    /// `true` if threshold reached
    pub status: bool,
}

/// Models the result of "getpeerinfo"
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetPeerInfoResult {
    /// Peer index
    pub id: u64,
    /// The IP address and port of the peer
    // TODO: use a type for addr
    pub addr: String,
    /// Bind address of the connection to the peer
    // TODO: use a type for addrbind
    pub addrbind: String,
    /// Local address as reported by the peer
    // TODO: use a type for addrlocal
    pub addrlocal: String,
    /// The services offered
    // TODO: use a type for services
    pub services: String,
    /// Whether peer has asked us to relay transactions to it
    pub relaytxes: bool,
    /// The time in seconds since epoch (Jan 1 1970 GMT) of the last send
    pub lastsend: u64,
    /// The time in seconds since epoch (Jan 1 1970 GMT) of the last receive
    pub lastrecv: u64,
    /// The total bytes sent
    pub bytessent: u64,
    /// The total bytes received
    pub bytesrecv: u64,
    /// The connection time in seconds since epoch (Jan 1 1970 GMT)
    pub conntime: u64,
    /// The time offset in seconds
    pub timeoffset: u64,
    /// ping time (if available)
    pub pingtime: u64,
    /// minimum observed ping time (if any at all)
    pub minping: u64,
    /// ping wait (if non-zero)
    pub pingwait: u64,
    /// The peer version, such as 70001
    pub version: u64,
    /// The string version
    pub subver: String,
    /// Inbound (true) or Outbound (false)
    pub inbound: bool,
    /// Whether connection was due to `addnode`/`-connect` or if it was an
    /// automatic/inbound connection
    pub addnode: bool,
    /// The starting height (block) of the peer
    pub startingheight: u64,
    /// The ban score
    pub banscore: i64,
    /// The last header we have in common with this peer
    pub synced_headers: u64,
    /// The last block we have in common with this peer
    pub synced_blocks: u64,
    /// The heights of blocks we're currently asking from this peer
    pub inflight: Vec<u64>,
    /// Whether the peer is whitelisted
    pub whitelisted: bool,
    /// The total bytes sent aggregated by message type
    // TODO: use a type for bytessent_per_msg
    pub bytessent_per_msg: Value,
    /// The total bytes received aggregated by message type
    // TODO: use a type for bytesrecv_per_msg
    pub bytesrecv_per_msg: Value,
}

/// Models the result of "estimatesmartfee"
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EstimateSmartFeeResult {
    /// Estimate fee rate in BTC/kB.
    pub feerate: Option<Value>,
    /// Errors encountered during processing.
    pub errors: Option<Vec<String>>,
    /// Block number where estimate was found.
    pub blocks: i64,
}

/// Models the result of "waitfornewblock", and "waitforblock"
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct BlockRef {
    pub hash: sha256d::Hash,
    pub height: u64,
}

// Custom types for input arguments.

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum EstimateMode {
    Unset,
    Economical,
    Conservative,
}

/// A wrapper around bitcoin::SigHashType that will be serialized
/// according to what the RPC expects.
pub struct SigHashType(bitcoin::SigHashType);

impl From<bitcoin::SigHashType> for SigHashType {
    fn from(sht: bitcoin::SigHashType) -> SigHashType {
        SigHashType(sht)
    }
}

impl serde::Serialize for SigHashType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self.0 {
            bitcoin::SigHashType::All => "ALL",
            bitcoin::SigHashType::None => "NONE",
            bitcoin::SigHashType::Single => "SINGLE",
            bitcoin::SigHashType::AllPlusAnyoneCanPay => "ALL|ANYONECANPAY",
            bitcoin::SigHashType::NonePlusAnyoneCanPay => "NONE|ANYONECANPAY",
            bitcoin::SigHashType::SinglePlusAnyoneCanPay => "SINGLE|ANYONECANPAY",
        })
    }
}

// Used for createrawtransaction argument.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRawTransactionInput {
    pub txid: sha256d::Hash,
    pub vout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_position: Option<u32>,
    #[serde(rename = "change_type", skip_serializing_if = "Option::is_none")]
    pub change_type: Option<AddressType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_watching: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_unspents: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_rate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtract_fee_from_outputs: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaceable: Option<bool>,
    #[serde(rename = "conf_target", skip_serializing_if = "Option::is_none")]
    pub conf_target: Option<u32>,
    #[serde(rename = "estimate_mode", skip_serializing_if = "Option::is_none")]
    pub estimate_mode: Option<EstimateMode>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FundRawTransactionResult {
    #[serde(with = "::serde_hex")]
    pub hex: Vec<u8>,
    #[serde(with = "bitcoin::util::amount::serde::as_btc")]
    pub fee: Amount,
    #[serde(rename = "changepos")]
    pub change_position: i32,
}

impl FundRawTransactionResult {
    pub fn transaction(&self) -> Result<Transaction, encode::Error> {
        encode::deserialize(&self.hex)
    }
}

// Used for signrawtransaction argument.
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignRawTransactionInput {
    pub txid: sha256d::Hash,
    pub vout: u32,
    pub script_pub_key: Script,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeem_script: Option<Script>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bitcoin::util::amount::serde::as_btc::opt"
    )]
    pub amount: Option<Amount>,
}

/// Used to represent an address type.
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum AddressType {
    Legacy,
    P2shSegwit,
    Bech32,
}

/// Used to represent arguments that can either be an address or a public key.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PubKeyOrAddress<'a> {
    Address(&'a Address),
    PubKey(&'a PublicKey),
}

impl<'a> serde::Serialize for PubKeyOrAddress<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            PubKeyOrAddress::Address(a) => serde::Serialize::serialize(a, serializer),
            PubKeyOrAddress::PubKey(k) => serde::Serialize::serialize(k, serializer),
        }
    }
}

// Custom deserializer functions.

/// deserialize_pubkey deserializes a secp256k1 pubkey from a string
/// PublicKey type.
fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    PublicKey::from_str(s.as_ref())
        .map_err(|_| D::Error::custom(&format!("error parsing pubkey: {}", s)))
}

/// deserialize_amount deserializes a BTC-denominated floating point Bitcoin amount into the
/// Amount type.
fn deserialize_amount<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Amount::from_btc(f64::deserialize(deserializer)?).unwrap())
}

/// deserialize_amount_opt deserializes a BTC-denominated floating point Bitcoin amount into an
/// Option of the Amount type.
fn deserialize_amount_opt<'de, D>(deserializer: D) -> Result<Option<Amount>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Amount::from_btc(f64::deserialize(deserializer)?).unwrap()))
}

fn deserialize_difficulty<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = f64::deserialize(deserializer)?.to_string();
    let real = match s.split('.').nth(0) {
        Some(r) => r,
        None => return Err(D::Error::custom(&format!("error parsing difficulty: {}", s))),
    };
    BigUint::from_str(real)
        .map_err(|_| D::Error::custom(&format!("error parsing difficulty: {}", s)))
}

/// deserialize_hex_array_opt deserializes a vector of hex-encoded byte arrays.
fn deserialize_hex_array_opt<'de, D>(deserializer: D) -> Result<Option<Vec<Vec<u8>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    //TODO(stevenroose) Revisit when issue is fixed:
    // https://github.com/serde-rs/serde/issues/723

    let v: Vec<String> = Vec::deserialize(deserializer)?;
    let mut res = Vec::new();
    for h in v.into_iter() {
        res.push(hex::decode(h).map_err(D::Error::custom)?);
    }
    Ok(Some(res))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::hex::FromHex;
    use serde_json;

    macro_rules! hex {
        ($h:expr) => {
            hex::decode(&$h).unwrap()
        };
    }

    macro_rules! deserializer {
        ($j:expr) => {
            &mut serde_json::Deserializer::from_str($j)
        };
    }

    macro_rules! hash {
        ($h:expr) => {
            sha256d::Hash::from_hex($h).unwrap()
        };
    }

    macro_rules! addr {
        ($a:expr) => {
            Address::from_str($a).unwrap()
        };
    }

    macro_rules! script {
        ($s:expr) => {
            serde_json::from_str(&format!(r#""{}""#, $s)).unwrap()
        };
    }

    #[test]
    fn test_AddMultiSigAddressResult() {
        let expected = AddMultiSigAddressResult {
            address: addr!("2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E"),
            redeem_script: script!("51210330aa51b444e2bac981235a0056112385057492c6cd06936af410c5af27c1f9462103dae74774a6cd35d948ee60bc7a1b35fdaed7b54698762e963e3677f795c7ad2a52ae"),
        };
        let json = r#"
            {
              "address": "2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E",
              "redeemScript": "51210330aa51b444e2bac981235a0056112385057492c6cd06936af410c5af27c1f9462103dae74774a6cd35d948ee60bc7a1b35fdaed7b54698762e963e3677f795c7ad2a52ae"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_GetBlockResult() {
        let expected = GetBlockResult {
            hash: hash!("000000006c02c8ea6e4ff69651f7fcde348fb9d557a06e6957b65552002a7820"),
            confirmations: 1414401,
            size: 190,
            strippedsize: Some(190),
            weight: 760,
            height: 2,
            version: 1,
            version_hex: Some(hex!("00000001")),
            merkleroot: hash!("20222eb90f5895556926c112bb5aa0df4ab5abc3107e21a6950aec3b2e3541e2"),
            tx: vec![hash!("20222eb90f5895556926c112bb5aa0df4ab5abc3107e21a6950aec3b2e3541e2")],
            time: 1296688946,
            mediantime: Some(1296688928),
            nonce: 875942400,
            bits: "1d00ffff".into(),
            difficulty: 1u64.into(),
            chainwork: hex!("0000000000000000000000000000000000000000000000000000000300030003"),
            contracthash: hash!("43d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"),
            attestationhash: hash!(
                "53d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"
            ),
            mappinghash: hash!("63d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"),
            previousblockhash: Some(hash!(
                "00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206"
            )),
            nextblockhash: Some(hash!(
                "000000008b896e272758da5297bcd98fdc6d97c9b765ecec401e286dc1fdbe10"
            )),
        };
        let json = r#"
            {
              "hash": "000000006c02c8ea6e4ff69651f7fcde348fb9d557a06e6957b65552002a7820",
              "confirmations": 1414401,
              "strippedsize": 190,
              "size": 190,
              "weight": 760,
              "height": 2,
              "version": 1,
              "versionHex": "00000001",
              "merkleroot": "20222eb90f5895556926c112bb5aa0df4ab5abc3107e21a6950aec3b2e3541e2",
              "tx": [
                "20222eb90f5895556926c112bb5aa0df4ab5abc3107e21a6950aec3b2e3541e2"
              ],
              "time": 1296688946,
              "mediantime": 1296688928,
              "nonce": 875942400,
              "bits": "1d00ffff",
              "difficulty": 1,
              "chainwork": "0000000000000000000000000000000000000000000000000000000300030003",
              "contracthash": "43d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "attestationhash": "53d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "mappinghash": "63d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "previousblockhash": "00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206",
              "nextblockhash": "000000008b896e272758da5297bcd98fdc6d97c9b765ecec401e286dc1fdbe10"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_GetBlockHeaderResult() {
        let expected = GetBlockHeaderResult {
            hash: hash!("00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765"),
            confirmations: 29341,
            height: 1384958,
            version: 536870912,
            version_hex: Some(hex!("20000000")),
            merkleroot: hash!("33d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"),
            time: 1534935138,
            mediantime: Some(1534932055),
            nonce: 871182973,
            bits: "1959273b".into(),
            difficulty: 48174374u64.into(),
            chainwork: hex!("0000000000000000000000000000000000000000000000a3c78921878ecbafd4"),
            contracthash: hash!("43d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"),
            attestationhash: hash!(
                "53d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"
            ),
            mappinghash: hash!("63d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7"),
            previousblockhash: Some(hash!(
                "000000000000002937dcaffd8367cfb05cd9ef2e3bd7a081de82696f70e719d9"
            )),
            nextblockhash: Some(hash!(
                "00000000000000331dddb553312687a4be62635ad950cde36ebc977c702d2791"
            )),
        };
        let json = r#"
            {
              "hash": "00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765",
              "confirmations": 29341,
              "height": 1384958,
              "version": 536870912,
              "versionHex": "20000000",
              "merkleroot": "33d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "time": 1534935138,
              "mediantime": 1534932055,
              "nonce": 871182973,
              "bits": "1959273b",
              "difficulty": 48174374.44122773,
              "chainwork": "0000000000000000000000000000000000000000000000a3c78921878ecbafd4",
              "contracthash": "43d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "attestationhash": "53d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "mappinghash": "63d8a6f622182a4e844022bbc8aa51c63f6476708ad5cc5c451f2933753440d7",
              "previousblockhash": "000000000000002937dcaffd8367cfb05cd9ef2e3bd7a081de82696f70e719d9",
              "nextblockhash": "00000000000000331dddb553312687a4be62635ad950cde36ebc977c702d2791"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_GetMiningInfoResult() {
        let expected = GetMiningInfoResult {
            blocks: 1415011,
            currentblockweight: Some(0),
            currentblocktx: Some(0),
            difficulty: 1u32.into(),
            networkhashps: 11970022568515.56,
            pooledtx: 110,
            chain: "test".into(),
            warnings: "Warning: unknown new rules activated (versionbit 28)".into(),
        };
        let json = r#"
            {
              "blocks": 1415011,
              "currentblockweight": 0,
              "currentblocktx": 0,
              "difficulty": 1,
              "networkhashps": 11970022568515.56,
              "pooledtx": 110,
              "chain": "test",
              "warnings": "Warning: unknown new rules activated (versionbit 28)"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());

        let expected = GetMiningInfoResult {
            blocks: 585966,
            currentblockweight: None,
            currentblocktx: None,
            difficulty: "9064159826491".parse().unwrap(),
            networkhashps: 5.276674407862246e+19,
            pooledtx: 48870,
            chain: "main".into(),
            warnings: "".into(),
        };
        let json = r#"
            {
              "blocks": 585966,
              "difficulty": 9064159826491.41,
              "networkhashps": 5.276674407862246e+19,
              "pooledtx": 48870,
              "chain": "main",
              "warnings": ""
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_SendAnyToAddressResult() {
        let expected_single = SendAnyToAddressResult::Txid(hash!(
            "4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"
        ));
        let json = r#""4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91""#;
        assert_eq!(expected_single, serde_json::from_str(json).unwrap());
        let expected_many = SendAnyToAddressResult::Txids(vec![
            hash!("5a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
            hash!("6a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
            hash!("7a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
        ]);
        let json = r#"["5a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91","6a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91","7a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"]"#;
        assert_eq!(expected_many, serde_json::from_str(json).unwrap());
    }

    //TODO(stevenroose) coinbase variant
    #[test]
    fn test_GetRawTransactionResult() {
        let expected = GetRawTransactionResult {
			in_active_chain: None,
			hex: hex!("020000000001f6d59ba2e098a2a2eaecf06b02aa0773773449caf62bd4e9f17cdb9b0d679954000000006b483045022100c74ee0dd8f3f6c909635f7a2bb8dd2052e3547f94a520cdba2aa12668059dae302204306e11033f18f65560a52a860b098e7df0fa7d35350d16f1c5a86e2da2ae37e012102b672f428ad984563c0dec80b3912fcad871338545df1538fe26c390826fbb4b2000000000101f80bb0038f482243202f0b2dcf88d9b4e7f930a48a3fcdc003af76b1f9d60e63010000000005f5c92c00a06a2006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f1976a914bedb324be05d1a1254afeb3e7ef40fea0368bc1e88ac2102e25e582ac1adc69f168aa7dbf0a97341421e10b22c659927de24fdac6e9f1fae4101a48fe52775701556a4a2dbf3d95c0c13845bbf87271e745b1c454f8ebcb5cd4792a4139f419f192ca6e389531d46fa5857f2c109dfe4003ad8b2ce504b488bed00000000"),
			txid: hash!("4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
			hash: hash!("4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
			size: 226,
			vsize: 226,
			version: 2,
			locktime: 1384957,
			vin: vec![GetRawTransactionResultVin{
				txid: Some(hash!("f04a336cb0fac5611e625827bd89e0be5dd2504e6a98ecbfaa5fcf1528d06b58")),
				vout: Some(0),
				script_sig: serde::export::Some(GetRawTransactionResultVinScriptSig{asm:
                                        "3045022100e85425f6d7c589972ee061413bcf08dc8c8e589ce37b217535a42af924f0e4d602205c9ba9cb14ef15513c9d946fa1c4b797883e748e8c32171bdf6166583946e35c[ALL] 03dae30a4d7870cd87b45dd53e6012f71318fdd059c1c2623b8cc73f8af287bb2d".into(),
                                    hex:
                                        hex!("483045022100e85425f6d7c589972ee061413bcf08dc8c8e589ce37b217535a42af924f0e4d602205c9ba9cb14ef15513c9d946fa1c4b797883e748e8c32171bdf6166583946e35c012103dae30a4d7870cd87b45dd53e6012f71318fdd059c1c2623b8cc73f8af287bb2d"),}),
				sequence: 4294967294,
				txinwitness: None,
                coinbase: None,
                is_pegin: Some(false),

			}],
			vout: vec![GetRawTransactionResultVout{
				value: Amount::from_btc(44.98834461).unwrap(),
				n: 0,
                asset: hash!("9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"),
                assetlabel: None,
				script_pub_key: GetRawTransactionResultVoutScriptPubKey{
					asm: "OP_DUP OP_HASH160 f602e88b2b5901d8aab15ebe4a97cf92ec6e03b3 OP_EQUALVERIFY OP_CHECKSIG".into(),
					hex: hex!("76a914f602e88b2b5901d8aab15ebe4a97cf92ec6e03b388ac"),
					type_: Some("pubkeyhash".into()),
					addresses: Some(vec![addr!("CMAMyHorv18WKKTvg5Kifi8ap8CuSJUxXT")]),
                    req_sigs: Some(1),
				},
			}, GetRawTransactionResultVout{
				value: Amount::from_btc(1.0).unwrap(),
				n: 1,
                asset: hash!("9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad6"),
                assetlabel: Some("CBD".into()),
				script_pub_key: GetRawTransactionResultVoutScriptPubKey{
					asm: "OP_DUP OP_HASH160 687ffeffe8cf4e4c038da46a9b1d37db385a472d OP_EQUALVERIFY OP_CHECKSIG".into(),
					hex: hex!("76a914687ffeffe8cf4e4c038da46a9b1d37db385a472d88ac"),
                    type_: Some("pubkeyhash".into()),
					addresses: Some(vec![addr!("CfNiN39BCkzkj26bG2uXhKxN18DUeAGYkY")]),
                    req_sigs: None,
				},
			}],
			blockhash: Some(hash!("00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765")),
			confirmations: Some(29446),
			time: Some(1534935138),
			blocktime: Some(1534935138),
		};
        let json = r#"
			{
			  "txid": "4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91",
			  "hash": "4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91",
			  "version": 2,
			  "size": 226,
			  "vsize": 226,
			  "weight": 904,
			  "locktime": 1384957,
			  "vin": [
				{
				  "txid": "f04a336cb0fac5611e625827bd89e0be5dd2504e6a98ecbfaa5fcf1528d06b58",
				  "vout": 0,
				  "scriptSig": {
					"asm": "3045022100e85425f6d7c589972ee061413bcf08dc8c8e589ce37b217535a42af924f0e4d602205c9ba9cb14ef15513c9d946fa1c4b797883e748e8c32171bdf6166583946e35c[ALL] 03dae30a4d7870cd87b45dd53e6012f71318fdd059c1c2623b8cc73f8af287bb2d",
					"hex": "483045022100e85425f6d7c589972ee061413bcf08dc8c8e589ce37b217535a42af924f0e4d602205c9ba9cb14ef15513c9d946fa1c4b797883e748e8c32171bdf6166583946e35c012103dae30a4d7870cd87b45dd53e6012f71318fdd059c1c2623b8cc73f8af287bb2d"
				  },
				  "sequence": 4294967294,
                  "is_pegin": false
				}
			  ],
			  "vout": [
				{
				  "value": 44.98834461,
				  "n": 0,
                  "asset": "9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad5",
				  "scriptPubKey": {
					"asm": "OP_DUP OP_HASH160 f602e88b2b5901d8aab15ebe4a97cf92ec6e03b3 OP_EQUALVERIFY OP_CHECKSIG",
					"hex": "76a914f602e88b2b5901d8aab15ebe4a97cf92ec6e03b388ac",
					"reqSigs": 1,
					"type": "pubkeyhash",
					"addresses": [
					  "CMAMyHorv18WKKTvg5Kifi8ap8CuSJUxXT"
					]
				  }
				},
				{
				  "value": 1.00000000,
				  "n": 1,
                  "asset": "9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad6",
                  "assetlabel": "CBD",
				  "scriptPubKey": {
					"asm": "OP_DUP OP_HASH160 687ffeffe8cf4e4c038da46a9b1d37db385a472d OP_EQUALVERIFY OP_CHECKSIG",
					"hex": "76a914687ffeffe8cf4e4c038da46a9b1d37db385a472d88ac",
					"type": "pubkeyhash",
					"addresses": [
					  "CfNiN39BCkzkj26bG2uXhKxN18DUeAGYkY"
					]
				  }
				}
			  ],
			  "hex": "020000000001f6d59ba2e098a2a2eaecf06b02aa0773773449caf62bd4e9f17cdb9b0d679954000000006b483045022100c74ee0dd8f3f6c909635f7a2bb8dd2052e3547f94a520cdba2aa12668059dae302204306e11033f18f65560a52a860b098e7df0fa7d35350d16f1c5a86e2da2ae37e012102b672f428ad984563c0dec80b3912fcad871338545df1538fe26c390826fbb4b2000000000101f80bb0038f482243202f0b2dcf88d9b4e7f930a48a3fcdc003af76b1f9d60e63010000000005f5c92c00a06a2006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f1976a914bedb324be05d1a1254afeb3e7ef40fea0368bc1e88ac2102e25e582ac1adc69f168aa7dbf0a97341421e10b22c659927de24fdac6e9f1fae4101a48fe52775701556a4a2dbf3d95c0c13845bbf87271e745b1c454f8ebcb5cd4792a4139f419f192ca6e389531d46fa5857f2c109dfe4003ad8b2ce504b488bed00000000",
			  "blockhash": "00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765",
			  "confirmations": 29446,
			  "time": 1534935138,
			  "blocktime": 1534935138
			}
		"#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
        assert!(expected.transaction().is_ok());
        assert_eq!(
            expected.transaction().unwrap().input[0].previous_output.txid,
            hash!("5499670d9bdb7cf1e9d42bf6ca4934777307aa026bf0eceaa2a298e0a29bd5f6")
        );
        assert!(expected.vin[0].script_sig.as_ref().unwrap().script().is_ok());
        assert!(expected.vout[0].script_pub_key.script().is_ok());
    }

    #[test]
    fn test_GetTransactionResult() {
        let expected = GetTransactionResult {
            amount: Amount::from_btc(1.0).unwrap(),
            fee: None,
            info: WalletTxInfo {
                confirmations: 30104,
                blockhash: Some(hash!("00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765")),
                blockindex: Some(2028),
                blocktime: Some(1534935138),
                txid: hash!("4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91"),
                time: 1534934745,
                timereceived: 1534934745,
                bip125_replaceable: Bip125Replaceable::No,
            },
            details: vec![
                GetTransactionResultDetail {
                    address: addr!("2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E"),
                    category: GetTransactionResultDetailCategory::Receive,
                    amount: Amount::from_btc(1.0).unwrap(),
                    label: Some("".into()),
                    vout: 1,
                    fee: None,
                    abandoned: None,
                },
            ],
            hex: hex!("020000000001f6d59ba2e098a2a2eaecf06b02aa0773773449caf62bd4e9f17cdb9b0d679954000000006b483045022100c74ee0dd8f3f6c909635f7a2bb8dd2052e3547f94a520cdba2aa12668059dae302204306e11033f18f65560a52a860b098e7df0fa7d35350d16f1c5a86e2da2ae37e012102b672f428ad984563c0dec80b3912fcad871338545df1538fe26c390826fbb4b2000000000101f80bb0038f482243202f0b2dcf88d9b4e7f930a48a3fcdc003af76b1f9d60e63010000000005f5c92c00a06a2006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f1976a914bedb324be05d1a1254afeb3e7ef40fea0368bc1e88ac2102e25e582ac1adc69f168aa7dbf0a97341421e10b22c659927de24fdac6e9f1fae4101a48fe52775701556a4a2dbf3d95c0c13845bbf87271e745b1c454f8ebcb5cd4792a4139f419f192ca6e389531d46fa5857f2c109dfe4003ad8b2ce504b488bed00000000"),
        };
        let json = r#"
            {
              "amount": 1.00000000,
              "confirmations": 30104,
              "blockhash": "00000000000000039dc06adbd7666a8d1df9acf9d0329d73651b764167d63765",
              "blockindex": 2028,
              "blocktime": 1534935138,
              "txid": "4a5b5266e1750488395ac15c0376c9d48abf45e4df620777fe8cff096f57aa91",
              "walletconflicts": [
              ],
              "time": 1534934745,
              "timereceived": 1534934745,
              "bip125-replaceable": "no",
              "details": [
                {
                  "address": "2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E",
                  "category": "receive",
                  "amount": 1.00000000,
                  "label": "",
                  "vout": 1
                }
              ],
              "hex": "020000000001f6d59ba2e098a2a2eaecf06b02aa0773773449caf62bd4e9f17cdb9b0d679954000000006b483045022100c74ee0dd8f3f6c909635f7a2bb8dd2052e3547f94a520cdba2aa12668059dae302204306e11033f18f65560a52a860b098e7df0fa7d35350d16f1c5a86e2da2ae37e012102b672f428ad984563c0dec80b3912fcad871338545df1538fe26c390826fbb4b2000000000101f80bb0038f482243202f0b2dcf88d9b4e7f930a48a3fcdc003af76b1f9d60e63010000000005f5c92c00a06a2006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f1976a914bedb324be05d1a1254afeb3e7ef40fea0368bc1e88ac2102e25e582ac1adc69f168aa7dbf0a97341421e10b22c659927de24fdac6e9f1fae4101a48fe52775701556a4a2dbf3d95c0c13845bbf87271e745b1c454f8ebcb5cd4792a4139f419f192ca6e389531d46fa5857f2c109dfe4003ad8b2ce504b488bed00000000"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
        assert!(expected.transaction().is_ok());
    }

    #[test]
    fn test_GetTxOutResult() {
        let expected = GetTxOutResult {
			bestblock: hash!("000000000000002a1fde7234dc2bc016863f3d672af749497eb5c227421e44d5"),
			confirmations: 29505,
			value: Amount::from_btc(1.0).unwrap(),
			script_pub_key: GetRawTransactionResultVoutScriptPubKey{
				asm: "OP_DUP OP_HASH160 687ffeffe8cf4e4c038da46a9b1d37db385a472d OP_EQUALVERIFY OP_CHECKSIG".into(),
				hex: hex!("76a914687ffeffe8cf4e4c038da46a9b1d37db385a472d88ac"),
				type_: Some("pubkeyhash".into()),
				addresses: Some(vec![addr!("2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E")]),
                req_sigs: Some(1)
			},
			coinbase: false,
		};
        let json = r#"
			{
			  "bestblock": "000000000000002a1fde7234dc2bc016863f3d672af749497eb5c227421e44d5",
			  "confirmations": 29505,
			  "value": 1.00000000,
			  "scriptPubKey": {
				"asm": "OP_DUP OP_HASH160 687ffeffe8cf4e4c038da46a9b1d37db385a472d OP_EQUALVERIFY OP_CHECKSIG",
				"hex": "76a914687ffeffe8cf4e4c038da46a9b1d37db385a472d88ac",
				"type": "pubkeyhash",
				"addresses": [
				  "2dxmEBXc2qMYcLSKiDBxdEePY3Ytixmnh4E"
				],
                "reqSigs": 1
			  },
			  "coinbase": false
			}
		"#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
        println!("{:?}", expected.script_pub_key.script());
        assert!(expected.script_pub_key.script().is_ok());
    }

    #[test]
    fn test_GetRequestsResult() {
        let expected = GetRequestsResult {
            genesis_block: hash!(
                "967da0e138b1014173844ee0e4d557ff8a2463b14fcaeab18f6a63aa7c7e1d05"
            ),
            start_block_height: 105,
            end_block_height: 120,
            confirmed_block_height: 30,
            num_tickets: 50,
            decay_const: 10000,
            fee_percentage: 4,
            start_price: Amount::from_btc(0.01).unwrap(),
            auction_price: Amount::from_btc(0.008).unwrap(),
            txid: hash!("aabbccddeeff06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"),
        };
        let json = r#"
            {
                "genesisBlock": "967da0e138b1014173844ee0e4d557ff8a2463b14fcaeab18f6a63aa7c7e1d05",
                "confirmedBlockHeight": 30,
                "startBlockHeight": 105,
                "numTickets": 50,
                "decayConst": 10000,
                "feePercentage": 4,
                "endBlockHeight": 120,
                "startPrice": 0.01000000,
                "auctionPrice": 0.00800000,
                "txid": "aabbccddeeff06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_GetRequestBidsResult() {
        let expected = GetRequestBidsResult {
            genesis_block: hash!(
                "967da0e138b1014173844ee0e4d557ff8a2463b14fcaeab18f6a63aa7c7e1d05"
            ),
            start_block_height: 105,
            end_block_height: 120,
            confirmed_block_height: 30,
            num_tickets: 50,
            decay_const: 10000,
            fee_percentage: 4,
            start_price: Amount::from_btc(0.01).unwrap(),
            auction_price: Amount::from_btc(0.008).unwrap(),
            txid: hash!("aabbccddeeff06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"),
            bids: vec![GetRequestBidsResultBid {
                txid: hash!("11223554466f06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"),
                fee_pub_key: PublicKey::from_str(
                    "0268680737c76dabb801cb2204f57dbe4e4579e4f710cd67dc1b4227592c81e9b5",
                )
                .unwrap(),
            }],
        };
        let json = r#"
            {
                "genesisBlock": "967da0e138b1014173844ee0e4d557ff8a2463b14fcaeab18f6a63aa7c7e1d05",
                "confirmedBlockHeight": 30,
                "startBlockHeight": 105,
                "numTickets": 50,
                "decayConst": 10000,
                "feePercentage": 4,
                "endBlockHeight": 120,
                "startPrice": 0.01000000,
                "auctionPrice": 0.00800000,
                "txid": "aabbccddeeff06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5",
                "bids": [
                    {
                      "feePubKey": "0268680737c76dabb801cb2204f57dbe4e4579e4f710cd67dc1b4227592c81e9b5",
                      "txid": "11223554466f06acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"
                    }
                  ]
            }
        "#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    #[test]
    fn test_ListUnspentResult() {
        let expected = ListUnspentResultEntry {
            txid: hash!("5a6a5685c04974448c9c80fb170ae7ca20c02bddd97d306ac0314b3a12b85cba"),
            vout: 37,
            address: addr!("2drnhKrVLEbjgD4sBdmFkTByNjDCJpFqT4W"),
            account: Some("".into()),
            script_pub_key: script!("76a914be70510653867b1c648b43cfb3b0edf8420f08d788ac"),
            amount: Amount::from_btc(210000.0).unwrap(),
            amountcommitment: None,
            asset: hash!("9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad5"),
            assetcommitment: None,
            assetlabel: Some("CBT".into()),
            confirmations: 1,
            redeem_script: None,
            spendable: true,
            solvable: true,
            blinder: hash!("0000000000000000000000000000000000000000000000000000000000000000"),
            assetblinder: hash!("0000000000000000000000000000000000000000000000000000000000000000"),
        };
        let json = r#"
			{
                "txid": "5a6a5685c04974448c9c80fb170ae7ca20c02bddd97d306ac0314b3a12b85cba",
                "vout": 37,
                "address": "2drnhKrVLEbjgD4sBdmFkTByNjDCJpFqT4W",
                "account": "",
                "scriptPubKey": "76a914be70510653867b1c648b43cfb3b0edf8420f08d788ac",
                "amount": 210000.00000000,
                "asset": "9fda3adca5a106acec3378cac65698c8138f3531b274d34dad131d0423f5cad5",
                "assetlabel": "CBT",
                "confirmations": 1,
                "spendable": true,
                "solvable": true,
                "blinder": "0000000000000000000000000000000000000000000000000000000000000000",
                "assetblinder": "0000000000000000000000000000000000000000000000000000000000000000"
			}
		"#;
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }

    //TODO(stevenroose) test SignRawTransactionResult

    //TODO(stevenroose) test UTXO

    #[test]
    fn test_deserialize_pubkey() {
        let vectors = vec![
            (
                r#""0268680737c76dabb801cb2204f57dbe4e4579e4f710cd67dc1b4227592c81e9b5""#,
                PublicKey::from_str(
                    "0268680737c76dabb801cb2204f57dbe4e4579e4f710cd67dc1b4227592c81e9b5",
                )
                .unwrap(),
            ),
            (
                r#""0368680737c76dabb801cb2204f57e4e4579e4f710cd67dc1b4227592c81e9b5""#,
                PublicKey::from_str(
                    "0268680737c76dabb801cb2204f57dbe4e4579e4f710cd67dc1b4227592c81e9b5",
                )
                .unwrap(),
            ),
        ];
        let mut v = vectors[0];
        assert_eq!(deserialize_pubkey(deserializer!(v.0)).unwrap(), v.1);
        v = vectors[1];
        match deserialize_pubkey(deserializer!(v.0)) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), format!("error parsing pubkey: 0368680737c76dabb801cb2204f57e4e4579e4f710cd67dc1b4227592c81e9b5"))
        }
    }

    #[test]
    fn test_deserialize_amount() {
        let vectors = vec![
            ("0", Amount::from_sat(0)),
            ("1", Amount::from_sat(100000000)),
            ("1.00000001", Amount::from_sat(100000001)),
            ("10000000.00000001", Amount::from_sat(1000000000000001)),
        ];
        for vector in vectors.into_iter() {
            let d = deserialize_amount(deserializer!(vector.0)).unwrap();
            assert_eq!(d, vector.1);
        }
    }

    #[test]
    fn test_deserialize_amount_opt() {
        let vectors = vec![
            ("0", Some(Amount::from_sat(0))),
            ("1", Some(Amount::from_sat(100000000))),
            ("1.00000001", Some(Amount::from_sat(100000001))),
            ("10000000.00000001", Some(Amount::from_sat(1000000000000001))),
        ];
        for vector in vectors.into_iter() {
            let d = deserialize_amount_opt(deserializer!(vector.0)).unwrap();
            assert_eq!(d, vector.1);
        }
    }

    #[test]
    fn test_deserialize_difficulty() {
        let vectors = vec![
            ("1.0", 1u64.into()),
            ("0", 0u64.into()),
            ("123.12345", 123u64.into()),
            ("10000000.00000001", 10000000u64.into()),
        ];
        for vector in vectors.into_iter() {
            let d = deserialize_difficulty(deserializer!(vector.0)).unwrap();
            assert_eq!(d, vector.1);
        }
    }

    #[test]
    fn test_deserialize_hex_array_opt() {
        let vectors = vec![(r#"["0102","a1ff"]"#, Some(vec![vec![1, 2], vec![161, 255]]))];
        for vector in vectors.into_iter() {
            let d = deserialize_hex_array_opt(deserializer!(vector.0)).unwrap();
            assert_eq!(d, vector.1);
        }
    }
}
