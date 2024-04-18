#![allow(unreachable_patterns)]

pub mod obj;
pub mod mock;
pub mod node;

use core::net::SocketAddr;

use futures::Future;
use obj::{ReqMessage, RespMessage};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Endpoint {
    type Conn;
    type Err;

    fn connect(&self, domain: &str, addr: SocketAddr) -> impl Future<Output = Result<Self::Conn, Self::Err>>;
    fn accept(&self) -> impl Future<Output = Result<Self::Conn, Self::Err>>;
}

pub trait Connection {
    type Request: Request;

    type Read: AsyncRead;
    type Write: AsyncWrite;

    type StreamError;
    type ReqError;

    /// Waits for the next stream to be opened by the endpoint.
    fn next_raw(&self) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>>;
    /// Opens a raw stream.
    fn open_raw(&self) -> impl Future<Output = Result<Stream<Self::Write, Self::Read>, Self::StreamError>>;

    /// Waits for the next request from the endpoint.
    fn next_request(&self) -> impl Future<Output = Result<Self::Request, Self::ReqError>>;
    /// Sends a request to the endpoint.
    fn request(&self, req: ReqMessage) -> impl Future<Output = Result<RespMessage, Self::ReqError>>;
}

pub trait Request {
    type Error;

    fn request_msg(&self) -> &ReqMessage;
    fn respond(self, resp: RespMessage) -> impl Future<Output = Result<(), Self::Error>>;
}

pub struct Stream<Write, Read> {
    pub read: Read,
    pub write: Write,
}