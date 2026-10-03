//! Bound a JSON-RPC line before the SDK accumulates or deserializes it.
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};

pub struct Lines<R> {
    inner: R,
    limit: usize,
    current: usize,
    refused: bool,
}
impl<R> Lines<R> {
    pub fn new(inner: R, limit: usize) -> Self {
        Self {
            inner,
            limit,
            current: 0,
            refused: false,
        }
    }
}
impl<R: AsyncRead + Unpin> AsyncRead for Lines<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.refused {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP request exceeds frame limit",
            )));
        }
        let start = buffer.filled().len();
        match Pin::new(&mut self.inner).poll_read(cx, buffer) {
            Poll::Ready(Ok(())) => {
                for byte in &buffer.filled()[start..] {
                    self.current = self.current.saturating_add(1);
                    if self.current > self.limit {
                        self.refused = true;
                        buffer.set_filled(start);
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "MCP request exceeds frame limit",
                        )));
                    }
                    if *byte == b'\n' {
                        self.current = 0;
                    }
                }
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

pub fn cap(max_input: usize) -> usize {
    std::env::var("INKSCAPE_MCP_MAX_REQUEST_BYTES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        // A JSON string can use six wire bytes for a one-byte escaped SVG character.
        .unwrap_or_else(|| max_input.saturating_mul(6).saturating_add(1024 * 1024))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn exact_limit_multiple_lines_and_partial_reads_preserve_bytes() {
        let mut reader = Lines::new(&b"abc\nx\r\n123\n"[..], 4);
        let mut result = Vec::new();
        let mut byte = [0];
        while reader.read(&mut byte).await.unwrap() != 0 {
            result.push(byte[0]);
        }
        assert_eq!(result, b"abc\nx\r\n123\n");
    }

    #[tokio::test]
    async fn unterminated_line_refuses_without_waiting_for_eof_and_stays_refused() {
        let (mut writer, reader) = tokio::io::duplex(32);
        let mut limited = Lines::new(reader, 4);
        writer.write_all(b"abcd").await.unwrap();
        let mut buffer = [0; 4];
        assert_eq!(limited.read(&mut buffer).await.unwrap(), 4);
        writer.write_all(b"e").await.unwrap();
        let error = limited.read(&mut buffer).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        writer.write_all(b"\nx\n").await.unwrap();
        assert!(limited.read(&mut buffer).await.is_err());
    }
}
