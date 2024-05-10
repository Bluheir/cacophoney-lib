mod message;
mod signables;

use core::net::IpAddr;

use arcstr::ArcStr;
pub use message::*;
use serde::{Deserialize, Serialize};
pub use signables::*;

use crate::crypto::{KeyTriad, PublicKey};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyReq {
    pub keys: Vec<KeyTriad<SignedData>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyResp {}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PreIdentifyReq {}

/// A request that asks if the specified public keys have connected to the node.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct KeysExistsReq {
    /// The public keys.
    pub keys: Vec<PublicKey>,
    /// If a public key in `keys` has not connected to the node, notify the client when it connects.
    pub notify: bool,
}

/// A response to a [`KeysExistsReq`]. Returns the public keys that have connected to the node,
/// and the cryptographic proofs that they have connected.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct KeysExistsResp {
    pub triads: Vec<KeyTriad<SignedData>>,
}

/// A request that asks if a client can communicate with another client identifying as a public key.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CommunicationReq {
    /// The public key of the initiator.
    pub from: PublicKey,
    /// The public key the initiator wants to communicate with.
    pub to: PublicKey,
}

/// A request to list the IP addresses and domain names of the servers that are connected to this node.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ListConnectedServersReq {}

/// A response to a [`ListConnectedServersReq`]. Contains the IP addresses and domain names of the connected servers.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ListConnectedServersResp {
    pub servers: Vec<ConnectedServer>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ConnectedServer {
    /// The IP address of the connected server.
    pub ip: IpAddr,
    /// The domain name of the connected server.
    pub domain: ArcStr,
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
