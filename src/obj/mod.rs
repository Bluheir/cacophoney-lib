mod message;
mod signables;

pub use message::*;
pub use signables::*;
use serde::{Deserialize, Serialize};

use crate::crypto::{KeyTriad, PublicKey, Signature};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyReq {
    pub keys: Vec<KeyTriad<Signed>>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyResp { }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PreIdentifyReq { }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Hash)]
pub struct NodeInfo {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Hash)]
pub struct NodeInfoResp {
    /// If the versions are compatible with each other.
    pub compatible: bool,
    /// The node info sent in response.
    pub info: NodeInfo
}