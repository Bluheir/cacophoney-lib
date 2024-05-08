mod message;
mod signables;

pub use message::*;
use serde::{Deserialize, Serialize};
pub use signables::*;

use crate::crypto::{KeyTriad, PublicKey};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyReq {
    pub keys: Vec<KeyTriad<Signed>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyResp {}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PreIdentifyReq {}

/// A request that asks if the specified public keys have connected to the node.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct KeysExistsReq { pub keys: Vec<PublicKey> }

/// A response to a [`KeysExistsReq`]. Returns the public keys that have connected to the node,
/// and the cryptographic proofs that they have connected.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct KeysExistsResp {
    pub triads: Vec<KeyTriad<SignedData>>
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Hash,
)]
pub struct NodeInfo {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Hash,
)]
pub struct NodeInfoResp {
    /// If the versions are compatible with each other.
    pub compatible: bool,
    /// The node info sent in response.
    pub info: NodeInfo,
}
