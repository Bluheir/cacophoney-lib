use core::net::SocketAddr;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

use crate::obj::{ReqMessage, RespMessage};

#[derive(Debug)]
pub struct Context {
    nodes: RwLock<HashMap<SocketAddr, mpsc::Sender<Connection>>>,
}

#[derive(Debug)]
pub struct Endpoint {
    ctx: Arc<Context>,
    addr: SocketAddr,
    conns: Mutex<mpsc::Receiver<Connection>>,
}

impl Endpoint {
    pub async fn new(ctx: Arc<Context>, addr: SocketAddr) -> Self {
        let (send, recv) = mpsc::channel(32);

        let mut nodes = ctx.nodes.write().await;
        nodes.insert(addr, send);
        drop(nodes);

        Self {
            ctx,
            addr,
            conns: Mutex::new(recv)
        }
    }
}

impl crate::Endpoint for Endpoint {
    type Conn = Connection;
    type Err = Infallible;

    async fn connect(&self, _domain: &str, addr: SocketAddr) -> Result<Self::Conn, Self::Err> {
        let nodes = self.ctx.nodes.read().await;
        let node = nodes.get(&addr).unwrap().clone();

        let pair = connection_pair();
        node.send(pair.0).await.unwrap();

        Ok(pair.1)
    }

    async fn accept(&self) -> Result<Self::Conn, Self::Err> {
        let mut conn = self.conns.lock().await;
        Ok(conn.recv().await.unwrap())
    }
}

pub struct Request {
    pub resp: oneshot::Sender<RespMessage>,
    pub req: ReqMessage,
}

impl crate::Request for Request {
    type Error = Infallible;

    fn request_msg(&self) -> &ReqMessage {
        &self.req
    }
    async fn respond(self, resp: RespMessage) -> Result<(), Self::Error> {
        let _ = self.resp.send(resp);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Connection {
    pub send_req: mpsc::Sender<Request>,
    pub recv_req: Mutex<mpsc::Receiver<Request>>,
}

pub fn connection_pair() -> (Connection, Connection) {
    let (send1, recv1) = mpsc::channel(32);
    let (send2, recv2) = mpsc::channel(32);

    (Connection {
        send_req: send1,
        recv_req: Mutex::new(recv2),
    }, Connection {
        send_req: send2,
        recv_req: Mutex::new(recv1),
    })
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    #[error("stream closed")]
    Closed
}

impl crate::Connection for Connection {
    type Request = Request;

    type Read = tokio::net::TcpStream;
    type Write = tokio::net::TcpStream;

    type StreamError = Infallible;
    type ReqError = Error;

    async fn next_raw(&self) -> Result<crate::Stream<Self::Write, Self::Read>, Self::StreamError> {
        todo!()
    }

    async fn open_raw(&self) -> Result<crate::Stream<Self::Write, Self::Read>, Self::StreamError> {
        todo!()
    }

    async fn next_request(&self) -> Result<Self::Request, Self::ReqError> {
        let mut reqs = self.recv_req.lock().await;
        match reqs.recv().await {
            Some(v) => Ok(v),
            None => Err(Error::Closed)
        }

    }

    async fn request(&self, req: ReqMessage) -> Result<RespMessage, Self::ReqError> {
        let (resp, recv) = oneshot::channel();
        let req = Request {
            resp,
            req,
        };

        match self.send_req.send(req).await {
            Ok(_) => {},
            Err(_) => return Err(Error::Closed),
        }

        Ok(recv.await.unwrap())
    }
}