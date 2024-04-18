use serde::Serialize;
use thiserror::Error;

use std::error::Error as StdError;

use crate::obj::InvalidTypeError;

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
