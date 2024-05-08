#![allow(unreachable_patterns)]

pub mod crypto;
pub mod mock;
pub mod node;
pub mod obj;
mod utils;

#[cfg(test)]
mod tests;

use core::net::SocketAddr;
use std::{error::Error as StdError, sync::Arc};

use futures::Future;
use obj::{ReqMessage, RespMessage};
use tokio::io::{AsyncRead, AsyncWrite};

pub const CURRENT_VERSION: u32 = 0;

pub trait Endpoint {
    type Conn: Connection;
    type Error: StdError;

    fn connect(
        &self,
        domain: &str,
        addr: SocketAddr,
    ) -> impl Future<Output = Result<Self::Conn, Self::Error>>;
    fn accept(&self) -> impl Future<Output = Result<Self::Conn, Self::Error>>;
}

pub trait Connection {
    type Responder: Request;

    type Read: AsyncRead;
    type Write: AsyncWrite;

    type StreamError: StdError;
    type ReqError: StdError;

    /// Waits for the next stream to be opened by the endpoint.
    fn next_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>>;
    /// Opens a raw stream.
    fn open_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>>;

    /// Waits for the next request from the endpoint.
    fn next_request(
        &self,
    ) -> impl Future<Output = Result<(ReqMessage, Self::Responder), Self::ReqError>>;
    /// Sends a request to the endpoint.
    fn request(&self, req: ReqMessage)
        -> impl Future<Output = Result<RespMessage, Self::ReqError>>;
}

impl<T: Connection> Connection for &T {
    type Responder = T::Responder;

    type Read = T::Read;
    type Write = T::Write;

    type StreamError = T::StreamError;
    type ReqError = T::ReqError;

    fn next_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>> {
        (*self).next_raw()
    }
    fn open_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>> {
        (*self).open_raw()
    }
    fn next_request(
        &self,
    ) -> impl Future<Output = Result<(ReqMessage, Self::Responder), Self::ReqError>> {
        (*self).next_request()
    }
    fn request(
        &self,
        req: ReqMessage,
    ) -> impl Future<Output = Result<RespMessage, Self::ReqError>> {
        (*self).request(req)
    }
}
impl<T: Connection> Connection for Arc<T> {
    type Responder = T::Responder;

    type Read = T::Read;
    type Write = T::Write;

    type StreamError = T::StreamError;
    type ReqError = T::ReqError;

    fn next_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>> {
        (**self).next_raw()
    }
    fn open_raw(
        &self,
    ) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>> {
        (**self).open_raw()
    }
    fn next_request(
        &self,
    ) -> impl Future<Output = Result<(ReqMessage, Self::Responder), Self::ReqError>> {
        (**self).next_request()
    }
    fn request(
        &self,
        req: ReqMessage,
    ) -> impl Future<Output = Result<RespMessage, Self::ReqError>> {
        (**self).request(req)
    }
}

pub trait Request {
    type Error: StdError;

    fn respond(self, resp: RespMessage) -> impl Future<Output = Result<(), Self::Error>>;
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Default,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Stream<Write, Read> {
    pub write: Write,
    pub read: Read,
}
