use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto::{hash, HashMsg, ToHashMsg};

/// The size (in bytes) of the nonce.
pub const SALT_SIZE: usize = 16;

#[derive(Debug, Error)]
pub enum SignedConvertError {
    #[error("{}", .0)]
    JsonError(#[from] serde_json::Error),
    #[error("{}", .0)]
    CborError(#[from] serde_cbor::Error),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CachedSigned<T> {
    pub signable: Signable<T>,
    pub value: SignedData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[serde(tag = "format", content = "signed")]
pub enum SignedData {
    #[serde(rename = "JSON")]
    Json(ArcStr),
    #[serde(rename = "CBOR")]
    Cbor(Vec<u8>),
}
impl SignedData {
    pub fn to_signable<'a, T: Deserialize<'a>>(
        &'a self,
    ) -> Result<Signable<T>, SignedConvertError> {
        Ok(match self {
            SignedData::Json(json) => serde_json::from_str(json.as_str())?,
            SignedData::Cbor(cbor) => serde_cbor::from_slice(&cbor)?,
        })
    }
    pub fn to_cached<T>(self) -> Result<CachedSigned<T>, SignedConvertError>
    where
        for<'a> T: Deserialize<'a>,
    {
        Ok(CachedSigned {
            signable: self.to_signable()?,
            value: self,
        })
    }
}
impl ToHashMsg for &SignedData {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        match self {
            SignedData::Json(value) => hash(value),
            SignedData::Cbor(value) => hash(value),
        }
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

    #[serde(rename = "expireTime")]
    /// The expiration timestamp.
    pub expire_time: u64,
}
