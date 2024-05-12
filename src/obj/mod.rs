mod message;
mod signables;

use core::net::{IpAddr, SocketAddr};

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
pub struct ListConnectedServersReq {
    /// The maximum amount of connected servers to list. Is [`None`] if there is no limit.
    pub max: Option<u32>,
}

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


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    /// The domain name of this server.
    pub domain: ArcStr,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EndpointInfo {
    /// The server info of this connected endpoint, if they are a server.
    pub server_info: Option<ServerInfo>,
    /// The socket address of this connected endpoint.
    pub endpoint: SocketAddr,
}
impl EndpointInfo {
    pub const fn non_server(endpoint: SocketAddr) -> Self {
        Self {
            server_info: None,
            endpoint,
        }
    }
}