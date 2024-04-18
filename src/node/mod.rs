use core::net::SocketAddr;
use std::sync::{Arc, Weak};

pub mod error;

use crate::obj::*;
use crate::*;
use error::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerHandle {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<E: Endpoint> {
    pub endpoint: E,
    pub info: NodeInfo,
    pub handle: Option<Arc<ServerHandle>>,
}

impl<E: Endpoint> Node<E> {
    pub const fn client(endpoint: E, info: NodeInfo) -> Self {
        Self {
            endpoint,
            info,
            handle: None,
        }
    }
    pub fn server(endpoint: E, info: NodeInfo) -> Self {
        Self {
            endpoint,
            info,
            handle: Some(Default::default()),
        }
    }
    /// Gets the inner server handle from this node, if any.
    pub fn server_handle(&self) -> Option<&ServerHandle> {
        self.handle.as_ref().map(|v| &**v)
    }

    pub async fn accept(
        &self,
    ) -> Result<NodeConnection<E::Conn>, (Option<<E::Conn as Connection>::Responder>, ConnError<E::Error, <E::Conn as Connection>::ReqError>)> {
        // Wait for the next connection
        let conn = self
            .endpoint
            .accept()
            .await
            .map_err(|err| (None, ConnError::ConnectionErr(err)))?;

        let (req, responder) = conn.next_request().await.map_err(|v| (None, ConnError::RequestErr(v)))?;

        let info: NodeInfo = match req.try_into() {
            Ok(v) => v,
            Err(err) => return Err((Some(responder), ConnError::TypeErr(err)))
        };

        if info.api_version > crate::CURRENT_VERSION {
            return Err((Some(responder), ConnError::IncompatibleVersion(info.api_version)))
        }

        Ok(NodeConnection {
            conn,
            info,
            handle: self.handle.as_ref().map(|v| Arc::downgrade(v))
        })
    }

    pub async fn connect(
        &self,
        domain: &str,
        addr: SocketAddr,
    ) -> Result<NodeConnection<E::Conn>, ConnError<E::Error, <E::Conn as Connection>::ReqError>> {
        // Connect to the endpoint
        let conn = self
            .endpoint
            .connect(domain, addr)
            .await
            .map_err(ConnError::ConnectionErr)?;

        // Send our node info to the endpoint and receive their info in response
        let resp: NodeInfoResp = conn
            .request(self.info.clone().into())
            .await
            .map_err(ConnError::RequestErr)?
            .try_into()?;

        if !resp.compatible {
            // Return an error telling of the version incompatibilities
            return Err(ConnError::IncompatibleVersion(resp.info.api_version));
        }

        Ok(NodeConnection {
            conn,
            info: resp.info,
            handle: self.handle.as_ref().map(|v| Arc::downgrade(v)),
        })
    }
}

pub struct NodeConnection<C> {
    /// The inner connection.
    conn: C,
    info: NodeInfo,
    handle: Option<Weak<ServerHandle>>,
}
impl<C> NodeConnection<C> {}
