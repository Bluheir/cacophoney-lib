use arcstr::ArcStr;
use core::net::SocketAddr;
use futures::Future;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    convert::Infallible,
    error::Error as StdError,
    sync::{Arc, Weak},
};
use tokio::sync::RwLock;
use tower_async::Service;

pub mod error;
#[cfg(test)]
mod tests;

use crate::crypto::*;
use crate::obj::*;
use crate::utils;
use error::*;

pub trait OpenStream: Service<PublicKey, Error = <Self as OpenStream>::Err> {
    type Err: StreamOpenError;

    fn open_stream(
        &self,
        key: PublicKey,
    ) -> impl Future<Output = Result<Self::Response, Self::Err>> {
        self.call(key)
    }
}

pub trait Notify {
    type Err: StdError;

    /// Notify this client that the public key has connected.
    fn notify_connected(
        &self,
        triad: &KeyTriad<SignedData>,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send + Sync;
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

#[derive(Debug, Default)]
pub struct ServerHandle<C: ?Sized> {
    /// A map from a public key to a handle.
    key_to_endpoint: scc::HashMap<PublicKey, InboundHdl<C>>,
    /// Nodes connected to this endpoint that are also servers.
    connected_servers: RwLock<HashSet<InboundHdl<C>>>,
    /// Client handles that requested that they be notified when a public key connects to the node.
    notifications: scc::HashMap<PublicKey, HashSet<InboundHdl<C>>>,
}

impl<C: ?Sized> ServerHandle<C> {
    pub fn new() -> Self {
        Self {
            connected_servers: Default::default(),
            key_to_endpoint: Default::default(),
            notifications: Default::default(),
        }
    }
    pub fn new_hdl() -> Arc<Self> {
        Arc::new(Self::new())
    }
    pub async fn connect_server(&self, server_hdl: InboundHdl<C>) -> Result<(), InboundHdl<C>> {
        if server_hdl.info.server_info.is_none() {
            // this isn't a server handle, return an error
            return Err(server_hdl);
        }

        let mut connected_servers = self.connected_servers.write().await;

        if connected_servers.contains(&server_hdl) {
            return Err(server_hdl);
        }

        connected_servers.insert(server_hdl);
        Ok(())
    }
}

/// An endpoint that can be cloned
pub type InboundHdl<C> = Arc<InboundEndpoint<C>>;

#[derive(Debug)]
pub struct InboundEndpoint<C: ?Sized> {
    id: u64,
    server_hdl: Option<Weak<ServerHandle<C>>>,
    identify_data: RwLock<Option<IdentifyData>>,
    public_keys: RwLock<Vec<PublicKey>>,
    identities: scc::HashMap<PublicKey, KeyTriad<CachedSigned<IdentifyData>>>,
    info: EndpointInfo,
    conn: C,
}

impl<C: ?Sized> PartialEq for InboundEndpoint<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<C: ?Sized> Eq for InboundEndpoint<C> {}
impl<C: ?Sized> std::hash::Hash for InboundEndpoint<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

macro_rules! service_fn {
    ($fn_name:ident, $input:ty) => {
        pub fn $fn_name(
            &self,
            req: $input,
        ) -> impl Future<
            Output = Result<<Self as Service<$input>>::Response, <Self as Service<$input>>::Error>,
        > + '_
        where
            Self: Service<$input>,
        {
            self.call(req)
        }
    };
}
macro_rules! service_fn_hdl {
    ($fn_name:ident, $input:ty) => {
        pub async fn $fn_name(
            self: &Arc<Self>,
            req: $input,
        ) -> Result<<Arc<Self> as Service<$input>>::Response, <Arc<Self> as Service<$input>>::Error>
        where
            Arc<Self>: Service<$input>,
        {
            self.call(req).await
        }
    };
}

impl<C> InboundEndpoint<C> {
    pub fn client(id: u64, info: EndpointInfo, conn: C) -> Self {
        Self {
            id,
            conn,
            server_hdl: None,
            info,
            identify_data: Default::default(),
            public_keys: Default::default(),
            identities: Default::default(),
        }
    }
    pub fn client_hdl(id: u64, info: EndpointInfo, conn: C) -> Arc<Self> {
        Arc::new(Self::client(id, info, conn))
    }
    pub fn server(id: u64, info: EndpointInfo, server_hdl: Arc<ServerHandle<C>>, conn: C) -> Self {
        Self {
            id,
            info,
            server_hdl: Some(Arc::downgrade(&server_hdl)),
            identify_data: Default::default(),
            public_keys: Default::default(),
            identities: Default::default(),
            conn,
        }
    }
    pub fn server_hdl(
        id: u64,
        info: EndpointInfo,
        server_hdl: Arc<ServerHandle<C>>,
        conn: C,
    ) -> Arc<Self> {
        Arc::new(Self::server(id, info, server_hdl, conn))
    }

    /// Returns the id of this [`InboundEndpoint`]. Ids are assigned to each connected endpoint.
    pub fn id(&self) -> u64 {
        self.id
    }
    /// Returns the server info, if any, of this endpoint.
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.info.server_info.as_ref()
    }

    // service related functions:
    pub async fn pre_identify(&self, req: PreIdentifyReq) -> IdentifyData {
        self.call(req).await.unwrap()
    }
    service_fn!(list_connected, ListConnectedServersReq);
    service_fn!(communicate, CommunicationReq);
    service_fn_hdl!(identify, KeyTriad<SignedData>);
    service_fn_hdl!(keys_exists, KeysExistsReq);
}

impl<C: ?Sized> Service<ListConnectedServersReq> for InboundEndpoint<C> {
    type Response = ListConnectedServersResp;
    type Error = ListConnectedServersReqError;

    async fn call(&self, req: ListConnectedServersReq) -> Result<Self::Response, Self::Error> {
        let ref server_hdl = *self
            .server_hdl
            .as_ref()
            .ok_or(NotServerError)?
            .upgrade()
            .ok_or(ServerHdlDroppedError)?;

        let connected_servers = server_hdl.connected_servers.read().await;
        let mut servers = Vec::with_capacity(req.max.map(|value| value as usize).unwrap_or(connected_servers.len()));

        for (index, server) in connected_servers.iter().enumerate() {
            if Some(index as u32 + 1) == req.max {
                break;
            }

            let info = &server.info;
            servers.push(ConnectedServer {
                ip: info.endpoint.ip(),
                domain: info.server_info.as_ref().unwrap().domain.clone(),
            })
        }

        Ok(ListConnectedServersResp { servers })
    }
}
impl<C: ?Sized> Service<ListConnectedServersReq> for InboundHdl<C> {
    type Response = <InboundEndpoint<C> as Service<ListConnectedServersReq>>::Response;
    type Error = <InboundEndpoint<C> as Service<ListConnectedServersReq>>::Error;

    fn call(
        &self,
        req: ListConnectedServersReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        (&**self).call(req)
    }
}
impl<C: OpenStream + ?Sized> Service<CommunicationReq> for InboundEndpoint<C> {
    type Response = C::Response;
    type Error = CommunicationReqError<C::Err>;

    async fn call(&self, req: CommunicationReq) -> Result<Self::Response, Self::Error> {
        let ref server_hdl = *self
            .server_hdl
            .as_ref()
            .ok_or(NotServerError)?
            .upgrade()
            .ok_or(ServerHdlDroppedError)?;

        // check if this endpoint identified as the public key
        if !self.identities.contains_async(&req.from).await {
            return Err(Self::Error::InvalidPublicKey);
        }

        // get the handle that the initiator will communicate with
        let to_hdl = match server_hdl.key_to_endpoint.get_async(&req.to).await {
            Some(value) => value,
            None => return Err(Self::Error::CannotFindKey),
        };

        // open a stream to the endpoint
        Ok(to_hdl.conn.open_stream(req.from).await?)
    }
}
impl<C: OpenStream + ?Sized> Service<CommunicationReq> for InboundHdl<C> {
    type Response = <InboundEndpoint<C> as Service<CommunicationReq>>::Response;
    type Error = <InboundEndpoint<C> as Service<CommunicationReq>>::Error;

    fn call(
        &self,
        req: CommunicationReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        (&**self).call(req)
    }
}
impl<C: ?Sized> Service<KeysExistsReq> for InboundHdl<C> {
    type Response = KeysExistsResp;
    type Error = KeysExistsReqError;

    async fn call(&self, req: KeysExistsReq) -> Result<Self::Response, Self::Error> {
        let mut triads = Vec::with_capacity(req.keys.len());
        let ref server_hdl = *self
            .server_hdl
            .as_ref()
            .ok_or(NotServerError)?
            .upgrade()
            .ok_or(ServerHdlDroppedError)?;

        let notify_when_left = |key: PublicKey| async move {
            if !req.notify {
                return;
            }

            let entry = &mut *server_hdl.notifications.entry_async(key).await.or_default();
            // Add this handle to the notifiations map.
            entry.insert(self.clone());
        };

        for key in req.keys {
            let hdl = match server_hdl.key_to_endpoint.get_async(&key).await {
                Some(value) => value.clone(),
                None => {
                    notify_when_left(key).await;
                    continue;
                }
            };

            let triad = match hdl.identities.get_async(&key).await {
                Some(entry) => (*entry).clone(),
                None => {
                    notify_when_left(key).await;
                    continue;
                }
            };

            // map from KeyTriad<CachedSigned<IdentifyData>> to KeyTriad<SignedData>
            let triad = triad.map(|value| value.value);

            triads.push(triad)
        }

        Ok(KeysExistsResp { triads })
    }
}
impl<C: ?Sized> Service<PreIdentifyReq> for InboundEndpoint<C> {
    type Response = IdentifyData;
    type Error = Infallible;

    async fn call(&self, _req: PreIdentifyReq) -> Result<Self::Response, Self::Error> {
        // generate salt using RNG
        let mut salt = [0u8; SALT_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        drop(rng);

        let start_time = utils::now();
        let identify_data = IdentifyData {
            salt,
            start_time,
            // this expires in 5 seconds. add 5000 milliseconds.
            expire_time: start_time + 5000,
        };

        let mut identify_data_w = self.identify_data.write().await;
        *identify_data_w = Some(identify_data.clone());

        Ok(identify_data)
    }
}
impl<C: ?Sized> Service<PreIdentifyReq> for InboundHdl<C> {
    type Response = <InboundEndpoint<C> as Service<PreIdentifyReq>>::Response;
    type Error = <InboundEndpoint<C> as Service<PreIdentifyReq>>::Error;

    fn call(
        &self,
        req: PreIdentifyReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        (**self).call(req)
    }
}
impl<C: Notify + Send + Sync + 'static + ?Sized> Service<KeyTriad<SignedData>> for InboundHdl<C> {
    type Response = IdentifyResp;
    type Error = IdentifyReqError;

    async fn call(&self, triad: KeyTriad<SignedData>) -> Result<Self::Response, Self::Error> {
        let identify_data_r = self.identify_data.read().await;

        let identify_data = match *identify_data_r {
            Some(value) => value,
            None => return Err(IdentifyReqError::IdentifyDataInvalid),
        };

        let cached = triad.signed.clone().to_cached::<IdentifyData>()?;
        let value = &cached.signable;

        // Check the validity of the signature and the message type
        if value.msg_type != SignMessageType::Identify
            || !triad.public_key.valid(&cached.value, &triad.signature)
        {
            return Err(IdentifyReqError::SignatureInvalid);
        }

        // Check if the identify data is the same.
        if value.obj != identify_data {
            return Err(IdentifyReqError::IdentifyDataInvalid);
        }

        if utils::now() > value.obj.expire_time {
            return Err(IdentifyReqError::Expired);
        }

        let public_key = triad.public_key;
        let cached_triad = KeyTriad {
            public_key,
            signature: triad.signature,
            signed: cached,
        };

        let server_hdl = match &self.server_hdl {
            Some(weak) => {
                let server_hdl = match weak.upgrade() {
                    Some(value) => value,
                    None => return Err(ServerHdlDroppedError.into()),
                };

                let _ = server_hdl
                    .key_to_endpoint
                    .insert_async(public_key, self.clone())
                    .await;

                Some(server_hdl)
            }
            None => None,
        };

        // Add to identities
        match self
            .identities
            .insert_async(public_key, cached_triad.clone())
            .await
        {
            Ok(_) => {}
            Err(_) => return Err(IdentifyReqError::AlreadyIdentified),
        }

        // Notify endpoints that wanted to be notified when this public key connected.
        match server_hdl {
            Some(server_hdl) => {
                tokio::spawn(async move {
                    let endpoints =
                        match server_hdl.notifications.remove_async(&public_key).await {
                            Some(value) => value,
                            None => return,
                        }
                        .1;

                    for endpoint in endpoints.into_iter() {
                        // Fire and forget the notification
                        let _ = endpoint.conn.notify_connected(&triad).await;
                    }
                });
            }
            None => {}
        }

        // Add to vector for enumeration
        let mut public_keys = self.public_keys.write().await;
        public_keys.push(public_key);

        Ok(IdentifyResp {})
    }
}
