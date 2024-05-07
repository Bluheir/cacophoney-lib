use serde::Serialize;
use thiserror::Error;

use std::error::Error as StdError;

use crate::obj::{InvalidTypeError, SignedConvertError};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
pub enum ConnError<Conn: StdError, Req: StdError> {
    #[error("cannot connect to endpoint with error: {}", .0)]
    ConnectionErr(Conn),
    #[error("while receiving/requesting: {}", .0)]
    RequestErr(Req),
    #[error("incompatible version, provided version: {}", .0)]
    IncompatibleVersion(u32),
    #[error("{}", .0)]
    TypeErr(#[from] InvalidTypeError),
}

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
#[error("not a node")]
pub struct NotNodeError;

#[derive(Error, Debug)]
pub enum IdentifyReqError {
    /// This error happens when upgrading a [`Weak`] to an [`std::sync::Arc`] yields [`None`]
    #[error("all instances of the node handle were dropped")]
    NodeHdlDropped,
    /// A digital signature was invalid.
    #[error("signature invalid")]
    SignatureInvalid,
    #[error("identify data invalid")]
    IdentifyDataInvalid,
    #[error("identify data expired")]
    Expired,
    #[error("already identified key")]
    AlreadyIdentified,
    #[error("{}", .0)]
    ConvertErr(#[from] SignedConvertError)   
}