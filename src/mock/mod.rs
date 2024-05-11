mod stream;

use futures::Future;
pub use stream::*;
use tokio::sync::mpsc;

use crate::{crypto::KeyTriad, node::Notify, obj::SignedData};

pub struct MockNotify {
    send: mpsc::Sender<KeyTriad<SignedData>>,
}

impl Notify for MockNotify {
    type Err = mpsc::error::SendError<KeyTriad<SignedData>>;

    fn notify_connected(
        &self,
        triad: &KeyTriad<SignedData>,
    ) -> impl Future<Output = Result<(), Self::Err>> + Send + Sync {
        self.send.send(triad.clone())
    }
}
