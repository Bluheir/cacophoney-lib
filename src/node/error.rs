use serde::Serialize;
use thiserror::Error;

use std::error::Error as StdError;

use crate::obj::{InvalidTypeError, SignedConvertError};

/// This error happens when an endpoint starts a request that only a server can fulfill.
#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
#[error("not a node")]
pub struct NotServerError;

/// This error happens when upgrading the [`Weak`](`std::sync::Weak`) pointing to the server handle
/// to an [`Arc`](`std::sync::Arc`) yields [`None`].
#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
#[error("all instances of the node handle were dropped")]
pub struct ServerHdlDroppedError;

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

#[derive(Error, Debug)]
pub enum IdentifyReqError {
    /// Refer to [`ServerHdlDroppedError`].
    #[error("{}", .0)]
    ServerHdlDropped(#[from] ServerHdlDroppedError),
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
    ConvertErr(#[from] SignedConvertError),
}

#[derive(Error, Debug)]
pub enum KeysExistsReqError {
    /// Refer to [`NotServerError`].
    #[error("{}", .0)]
    NotServer(#[from] NotServerError),
    /// Refer to [`ServerHdlDroppedError`].
    #[error("{}", .0)]
    ServerHdlDropped(#[from] ServerHdlDroppedError),
}

/// An error type corresponding to a stream being opened to a connection.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamOpenErrorType {
    /// The endpoint declined a communication request, for whatever reason.
    #[error("the endpoint declined a communication request")]
    EndpointDeclined,
}
/// An error with a [`StreamOpenErrorType`].
pub trait StreamOpenError: StdError {
    /// Tries to convert this error to a [`StreamOpenErrorType`]. If this returns [`None`],
    /// then this error does not match any of the error types.
    fn error_type(&self) -> Option<StreamOpenErrorType>;
}

/// An error that can occur when an endpoint initiates a communication request to another public key.
#[derive(Error, Debug)]
pub enum CommunicationReqError<Err: StreamOpenError> {
    /// Refer to [`NotServerError`].
    #[error("{}", .0)]
    NotServer(#[from] NotServerError),
    /// Refer to [`ServerHdlDroppedError`].
    #[error("{}", .0)]
    ServerHdlDropped(#[from] ServerHdlDroppedError),
    #[error("the endpoint did not identify as the public key")]
    InvalidPublicKey,
    #[error("the initiator did not ")]
    CannotFindKey,
    #[error("{}", .0)]
    StreamOpenErr(#[from] Err),
}

/// An error that can occur when an endpoint initiates a communication request to another public key.
#[derive(Error, Debug)]
pub enum ListConnectedServersReqError {
    /// Refer to [`NotServerError`].
    #[error("{}", .0)]
    NotServer(#[from] NotServerError),
    /// Refer to [`ServerHdlDroppedError`].
    #[error("{}", .0)]
    ServerHdlDropped(#[from] ServerHdlDroppedError),
}
