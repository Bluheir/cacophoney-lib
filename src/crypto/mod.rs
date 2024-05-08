use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::obj::{IdentifyData, SignMessageType, Signable, Signed};

/// The size (in bytes) of a public key.
pub const PUBLIC_KEY_SIZE: usize = 33;

/// The size (in bytes) of a private key.
pub const PRIVATE_KEY_SIZE: usize = 32;

/// The size (in bytes) of a hash.
pub const HASH_SIZE: usize = 32;

/// The size (in bytes) of a signature.
pub const SIGNATURE_SIZE: usize = 64;

/// Computes the hash of a value
pub fn hash(bytes: impl AsRef<[u8]>) -> HashMsg {
    HashMsg(blake3::hash(bytes.as_ref()).into())
}

/// A signature.
#[repr(transparent)]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Signature(#[serde_as(as = "[_; SIGNATURE_SIZE]")] pub [u8; SIGNATURE_SIZE]);

/// A public key.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct PublicKey(#[serde_as(as = "[_; PUBLIC_KEY_SIZE]")] pub [u8; PUBLIC_KEY_SIZE]);

impl PublicKey {
    pub fn valid(&self, msg: impl ToHashMsg, signature: &Signature) -> bool {
        let pubkey = match libsecp256k1::PublicKey::parse_compressed(&self.0) {
            Ok(value) => value,
            _ => return false,
        };

        let hashmsg = msg.to_hash_msg();
        let msg = libsecp256k1::Message::parse(&hashmsg.as_ref().0);
        let signature = libsecp256k1::Signature::parse_overflowing(&signature.0);

        libsecp256k1::verify(&msg, &signature, &pubkey)
    }
}

/// A private key.
#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct PrivateKey(pub [u8; PRIVATE_KEY_SIZE]);

impl PrivateKey {
    pub fn derive_public(&self) -> PublicKey {
        let seckey = libsecp256k1::SecretKey::parse(&self.0).unwrap();
        PublicKey(libsecp256k1::PublicKey::from_secret_key(&seckey).serialize_compressed())
    }
    pub fn sign(&self, msg: impl ToHashMsg) -> Signature {
        let seckey = libsecp256k1::SecretKey::parse(&self.0).unwrap();

        let hashmsg = msg.to_hash_msg();
        let msg = libsecp256k1::Message::parse(&hashmsg.as_ref().0);

        Signature(libsecp256k1::sign(&msg, &seckey).0.serialize())
    }
}
/// A keypair.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyPair {
    pub public: PublicKey,
    pub private: PrivateKey,
}
impl KeyPair {
    pub fn derive_public(&self) -> PublicKey {
        self.public
    }
    pub fn sign(&self, msg: impl ToHashMsg) -> Signature {
        (&self.private).sign(msg)
    }
}

/// A message that can be signed, or verified. Is typically a hash of a value.
#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct HashMsg(pub [u8; HASH_SIZE]);

impl AsRef<HashMsg> for HashMsg {
    fn as_ref(&self) -> &HashMsg {
        self
    }
}

pub trait ToHashMsg {
    type Output: AsRef<HashMsg>;

    fn to_hash_msg(self) -> Self::Output;
}

impl ToHashMsg for HashMsg {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        self
    }
}
impl<'a, T: ?Sized> ToHashMsg for &&'a T
where
    &'a T: ToHashMsg,
{
    type Output = <&'a T as ToHashMsg>::Output;

    fn to_hash_msg(self) -> Self::Output {
        (*self).to_hash_msg()
    }
}
impl<'a> ToHashMsg for &'a HashMsg {
    type Output = &'a HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        self
    }
}
impl ToHashMsg for &[u8] {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        hash(self)
    }
}
impl<const N: usize> ToHashMsg for [u8; N] {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        hash(self)
    }
}
impl<const N: usize> ToHashMsg for &[u8; N] {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        hash(self)
    }
}
impl ToHashMsg for Vec<u8> {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        hash(self)
    }
}
impl ToHashMsg for &Vec<u8> {
    type Output = HashMsg;

    fn to_hash_msg(self) -> Self::Output {
        hash(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct KeyTriad<T> {
    #[serde(rename = "publicKey")]
    pub public_key: PublicKey,
    pub signature: Signature,
    pub signed: T,
}

impl KeyTriad<Signed> {
    pub fn gen_signed(key: &PrivateKey, identify: &IdentifyData, msg_type: SignMessageType) -> Self {
        let signable = Signable { msg_type, obj: identify };
        let ser = serde_cbor::to_vec(&signable).unwrap();

        KeyTriad {
            public_key: key.derive_public(),
            signature: key.sign(&ser),
            signed: Signed::cbor(ser),
        }
    }
}
