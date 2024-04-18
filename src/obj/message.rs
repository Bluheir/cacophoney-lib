use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::*;

pub trait ObjectType {
    fn object_type(&self) -> &'static str;
}

macro_rules! convert_impl {
    ($for:ty, $name:expr, $msg:ident, $variant:ident) => {
        impl ObjectType for $for {
            fn object_type(&self) -> &'static str {
                $name
            }
        }

        convert_impl!($for, $name, $msg, $variant, no_obj_impl);
    };
    ($for:ty, $name: expr, $msg:ident, $variant:ident, no_obj_impl) => {
        impl Into<$msg> for $for {
            fn into(self) -> $msg {
                $msg::$variant(self)
            }
        }
        impl TryFrom<$msg> for $for {
            type Error = InvalidTypeError;

            fn try_from(value: $msg) -> Result<Self, Self::Error> {
                match value {
                    $msg::$variant(v) => Ok(v),
                    value @ _ => Err(InvalidTypeError { expected: $name, received: value.object_type() })
                }
            }
        }
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Error)]
#[error("expected object type {expected} however received {received}.")]
pub struct InvalidTypeError {
    pub expected: &'static str,
    pub received: &'static str,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum ReqMessage {
    #[serde(rename = "NODE_INFO")]
    Connect(NodeInfo),
    #[serde(rename = "START_IDENTIFY")]
    StartIdentify(StartIdentifyReq),
    #[serde(rename = "IDENTIFY")]
    Identify(IdentifyReq)
}

impl ObjectType for ReqMessage {
    fn object_type(&self) -> &'static str {
        match self {
            Self::Connect(v) => v.object_type(),
            Self::Identify(v) => v.object_type(),
            Self::StartIdentify(v) => v.object_type(),
        }
    }
}
convert_impl!(NodeInfo, "NODE_INFO", ReqMessage, Connect);
convert_impl!(IdentifyReq, "IDENTIFY", ReqMessage, Identify);
convert_impl!(StartIdentifyReq, "START_IDENTIFY", ReqMessage, StartIdentify);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum RespMessage {
    #[serde(rename = "NODE_INFO")]
    Connect(NodeInfoResp),
    #[serde(rename = "IDENTIFY")]
    Identify(IdentifyResp)
}

impl ObjectType for RespMessage {
    fn object_type(&self) -> &'static str {
        match self {
            Self::Connect(v) => v.object_type(),
            Self::Identify(v) => v.object_type(),
        }
    }
}
convert_impl!(NodeInfoResp, "NODE_INFO", RespMessage, Connect);
convert_impl!(IdentifyResp, "IDENTIFY", RespMessage, Identify);