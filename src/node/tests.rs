use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::convert::Infallible;

use futures::Future;

use crate::crypto::PrivateKey;
use crate::node::{KeyTriad, ServerHandle};
use crate::obj::{KeysExistsReq, SignMessageType, Signable, SignedData};
use crate::{node::InboundEndpoint, obj::PreIdentifyReq};

use super::{EndpointInfo, Notify, PRIVATE_KEY_SIZE};

/// The private key used for the unit tests.
/// I do *NOT* recommend using this for anything other than tests.
const PRIVATE_KEY: [u8; PRIVATE_KEY_SIZE] = [
    59, 120, 176, 12, 17, 37, 95, 32, 64, 53, 178, 193, 44, 9, 148, 4, 187, 63, 144, 195, 132, 19,
    169, 115, 232, 229, 225, 77, 170, 4, 162, 75,
];

/// Endpoint info used for the unit tests.
const ENDPOINT_INFO: EndpointInfo = EndpointInfo::non_server(SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    51763,
));

struct DummyNotify;

impl Notify for DummyNotify {
    type Err = Infallible;

    fn notify_connected(
        &self,
        _triad: &KeyTriad<SignedData>,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send + Sync {
        async { unimplemented!() }
    }
}

#[tokio::test]
async fn keys_exists() {
    let key = PrivateKey::new(PRIVATE_KEY);
    let server_hdl = ServerHandle::new_hdl();
    let hdl = InboundEndpoint::server_hdl(0, ENDPOINT_INFO, server_hdl.clone(), DummyNotify);

    let identify = hdl.pre_identify(PreIdentifyReq {}).await;
    let triad = KeyTriad::gen_signed(&key, &identify, SignMessageType::Identify);

    hdl.identify(triad.clone()).await.unwrap();

    let mut keys_exists = hdl
        .keys_exists(KeysExistsReq {
            keys: vec![key.derive_public()],
            notify: false,
        })
        .await
        .unwrap();
    let first = keys_exists.triads.remove(0);

    assert_eq!(first, triad);
}

#[tokio::test]
async fn fake_signature() {
    let key = PrivateKey::new(PRIVATE_KEY);
    let server_hdl = ServerHandle::new_hdl();
    let hdl = InboundEndpoint::server_hdl(0, ENDPOINT_INFO, server_hdl.clone(), DummyNotify);

    let identify = hdl.pre_identify(PreIdentifyReq {}).await;

    let signable = Signable {
        msg_type: SignMessageType::Identify,
        obj: identify,
    };
    let ser = SignedData::Cbor(serde_cbor::to_vec(&signable).unwrap());

    let triad = KeyTriad {
        public_key: key.derive_public(),
        signed: ser,
        signature: crate::node::Signature([1u8; 64]),
    };

    assert!(hdl.identify(triad).await.is_err())
}
