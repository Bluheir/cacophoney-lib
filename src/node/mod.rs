use futures::Future;
use std::convert::Infallible;
use std::sync::{Arc, Weak};
use rand::RngCore;
use tokio::sync::RwLock;
use tower_async::Service;

#[cfg(test)]
mod tests;
pub mod error;

use error::*;
use crate::crypto::*;
use crate::obj::*;
use crate::utils;

#[derive(Debug, Default)]
pub struct ServerHandle {
    key_to_endpoint: scc::HashMap<PublicKey, InboundHdl>,
}

impl ServerHandle {
    pub fn new() -> Self {
        Self { key_to_endpoint: Default::default() }
    }
    pub fn new_hdl() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

/// An endpoint that can be cloned
pub type InboundHdl = Arc<InboundEndpoint>;

#[derive(Debug)]
pub struct InboundEndpoint {
    server_hdl: Option<Weak<ServerHandle>>,
    identify_data: RwLock<Option<IdentifyData>>,
    public_keys: RwLock<Vec<PublicKey>>,
    identities: scc::HashMap<PublicKey, KeyTriad<CachedSigned<IdentifyData>>>,
}

impl InboundEndpoint {
    pub fn client() -> Self {
        Self {
            server_hdl: None,
            identify_data: Default::default(),
            public_keys: Default::default(),
            identities: Default::default(),
        }
    }
    pub fn client_hdl() -> Arc<Self> {
        Arc::new(Self::client())
    }
    pub fn server(server_hdl: Arc<ServerHandle>) -> Self {
        Self {
            server_hdl: Some(Arc::downgrade(&server_hdl)),
            identify_data: Default::default(),
            public_keys: Default::default(),
            identities: Default::default(),
        }
    }
    pub fn server_hdl(server_hdl: Arc<ServerHandle>) -> Arc<Self> {
        Arc::new(Self::server(server_hdl))
    }
    pub async fn pre_identify(&self, req: PreIdentifyReq) -> IdentifyData {
        self.call(req).await.unwrap()
    }
    pub async fn identify(
        self: &InboundHdl,
        triad: KeyTriad<Signed>,
    ) -> Result<IdentifyResp, IdentifyReqError> {
        self.call(triad).await
    }
    pub async fn keys_exists(&self, req: KeysExistsReq) -> Result<KeysExistsResp, KeysExistsReqError> {
        self.call(req).await
    }
}

impl Service<KeysExistsReq> for InboundEndpoint {
    type Response = KeysExistsResp;
    type Error = KeysExistsReqError;

    async fn call(&self, req: KeysExistsReq) -> Result<Self::Response, Self::Error> {
        let mut triads = Vec::with_capacity(req.keys.len());
        let ref server_hdl = *self
            .server_hdl
            .as_ref()
            .ok_or(KeysExistsReqError::NotNode)?
            .upgrade()
            .ok_or(KeysExistsReqError::NodeHdlDropped)?;

        for key in req.keys {
            let hdl = match server_hdl.key_to_endpoint.get_async(&key).await {
                Some(value) => value.clone(),
                None => continue,
            };

            let triad = match hdl.identities.get_async(&key).await {
                Some(entry) => (*entry).clone(),
                None => continue,
            };

            // map from KeyTriad<CachedSigned<IdentifyData>> to KeyTriad<SignedData>
            let triad = triad.map(|value| value.value);
            
            triads.push(triad)
        }

        Ok(KeysExistsResp { triads })
    }
}
impl Service<PreIdentifyReq> for InboundEndpoint {
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
impl Service<PreIdentifyReq> for InboundHdl {
    type Response = <InboundEndpoint as Service<PreIdentifyReq>>::Response;
    type Error = <InboundEndpoint as Service<PreIdentifyReq>>::Error;

    fn call(
        &self,
        req: PreIdentifyReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        (**self).call(req)
    }
}
impl Service<KeyTriad<Signed>> for InboundHdl {
    type Response = IdentifyResp;
    type Error = IdentifyReqError;

    async fn call(&self, triad: KeyTriad<Signed>) -> Result<Self::Response, Self::Error> {
        let identify_data_r = self.identify_data.read().await;

        let identify_data = match *identify_data_r {
            Some(value) => value,
            None => return Err(IdentifyReqError::IdentifyDataInvalid),
        };

        let cached = triad.signed.to_cached::<IdentifyData>()?;
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
        let triad = KeyTriad {
            public_key,
            signature: triad.signature,
            signed: cached,
        };

        match &self.server_hdl {
            Some(weak) => {
                let hdl = match weak.upgrade() {
                    Some(value) => value,
                    None => return Err(IdentifyReqError::NodeHdlDropped),
                };

                let _ = hdl
                    .key_to_endpoint
                    .insert_async(public_key, self.clone())
                    .await;
            }
            None => {}
        }

        // Add to identities
        match self.identities.insert_async(public_key, triad).await {
            Ok(_) => {}
            Err(_) => return Err(IdentifyReqError::AlreadyIdentified),
        }

        // Add to vector for enumeration
        let mut public_keys = self.public_keys.write().await;
        public_keys.push(public_key);

        Ok(IdentifyResp {})
    }
}
