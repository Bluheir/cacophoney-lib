use std::ops::{Deref, DerefMut};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The size (in bytes) of the nonce.
pub const SALT_SIZE: usize = 16;

#[derive(Debug, Error)]
pub enum SignedConvertError {
    /// Both JSON and CBOR components provided were [`None`].
    #[error("both JSON and CBOR components provided were None")]
    BothNone,
    #[error("{}", .0)]
    JsonError(#[from] serde_json::Error),
    #[error("{}", .0)]
    CborError(#[from] serde_cbor::Error),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Signed {
    #[serde(rename = "JSON")]
    #[serde(default)]
    pub json: Option<ArcStr>,
    #[serde(rename = "CBOR")]
    #[serde(default)]
    pub cbor: Option<Vec<u8>>,
}
impl Signed {
    pub fn to_signable<'a, T: Deserialize<'a>>(
        &'a self,
    ) -> Result<Signable<T>, SignedConvertError> {
        Ok(match &self.cbor {
            Some(value) => serde_cbor::from_slice(&value)?,
            None => serde_json::from_str(match &self.json {
                Some(value) => value.as_str(),
                None => return Err(SignedConvertError::BothNone),
            })?,
        })
    }
}

/// A message that when converted to JSON/CBOR/another format, can be signed.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Signable<T> {
    #[serde(rename = "msgType")]
    pub msg_type: SignMessageType,
    pub obj: T,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[non_exhaustive]
pub enum SignMessageType {
    #[serde(rename = "IDENTIFY")]
    Identify,
}

/// Identify data sent from a node to the signer.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct IdentifyData {
    /// Nonce.
    pub salt: [u8; SALT_SIZE],
    /// The starting timestamp.
    #[serde(rename = "startTime")]
    pub start_time: u64,
}

/// Identify data additionally containing the expiry. Sent from the signer to the node.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct FullIdentifyData {
    #[serde(flatten)]
    pub identify_data: IdentifyData,
    /// The expiration date of this identify.
    pub expiry: u64,
}
impl Deref for FullIdentifyData {
    type Target = IdentifyData;

    fn deref(&self) -> &Self::Target {
        &self.identify_data
    }
}
impl DerefMut for FullIdentifyData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identify_data
    }
}
