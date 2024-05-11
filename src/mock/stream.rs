use futures::task::{Context, Poll};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    pin::Pin,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::mpsc,
};
use tokio_util::sync::PollSender;

#[inline]
fn shutdown_err() -> IoError {
    IoErrorKind::BrokenPipe.into()
}

pub enum MockWrite {
    Normal { send: PollSender<Vec<u8>> },
    Shutdown,
}
impl AsyncWrite for MockWrite {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        match &mut *self {
            MockWrite::Normal { send } => {
                match send.poll_reserve(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => {}
                        Err(_) => Err(shutdown_err())?,
                    },
                    Poll::Pending => return Poll::Pending,
                }

                match send.send_item(buf.to_owned()) {
                    Ok(_) => {}
                    Err(_) => Err(shutdown_err())?,
                }
                Poll::Ready(Ok(buf.len()))
            }
            MockWrite::Shutdown => Err(shutdown_err())?,
        }
    }
    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        match &mut *self {
            MockWrite::Shutdown => Err(IoError::new(IoErrorKind::BrokenPipe, "already shutdown"))?,
            _ => Poll::Ready(Ok(())),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(match &*self {
            MockWrite::Shutdown => Err(IoError::new(IoErrorKind::BrokenPipe, "already shutdown")),
            _ => {
                *self = MockWrite::Shutdown;
                Ok(())
            }
        })
    }
}

pub struct MockRead {
    recv: mpsc::Receiver<Vec<u8>>,
    buf: Vec<u8>,
    pos: usize,
}
impl MockRead {
    /// The amount of bytes to read.
    #[inline]
    fn to_read(&self) -> usize {
        self.buf.len() - self.pos
    }
}
impl AsyncRead for MockRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        if self.to_read() == 0 {
            let bytes = match self.recv.poll_recv(cx) {
                Poll::Ready(bytes) => bytes.ok_or_else(|| shutdown_err())?,
                Poll::Pending => return Poll::Pending,
            };

            self.buf.extend_from_slice(&bytes);
        }

        let remaining = buf.remaining();
        let amt = std::cmp::min(self.to_read(), remaining);

        buf.put_slice(&self.buf[self.pos..amt + self.pos]);
        self.pos += amt;

        if self.to_read() == 0 {
            self.pos = 0;
            self.buf.clear();
        }

        Poll::Ready(Ok(()))
    }
}

pub fn stream_pair(buffer: usize) -> (MockRead, MockWrite) {
    let (send, recv) = mpsc::channel(buffer);

    (
        MockRead {
            recv,
            buf: Vec::new(),
            pos: 0,
        },
        MockWrite::Normal {
            send: PollSender::new(send),
        },
    )
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::mock::stream_pair;

    #[tokio::test]
    async fn data_test() {
        let (mut read, mut write) = stream_pair(12);

        let _ = write.write(b"msg").await;
        let _ = write.write(&[]).await;

        let mut buf = Vec::new();
        let _ = read.read_to_end(&mut buf).await.unwrap();

        assert_eq!(&buf, b"msg")
    }
}
