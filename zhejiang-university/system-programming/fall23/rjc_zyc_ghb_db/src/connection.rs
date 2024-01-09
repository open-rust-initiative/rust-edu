use std::io::{self, Cursor};

use bytes::{BytesMut, Buf};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::frame::Frame;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),  // 4KB
        }
    }

    pub async fn read(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse()? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub async fn write(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::String(string) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(string.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(string) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(string.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
        }
        self.stream.flush().await
    }

    fn parse(&mut self) -> crate::Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(crate::frame::Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}