use std::io::{Error, ErrorKind};

use tokio::net::{ToSocketAddrs, TcpStream};

use crate::{Connection, Frame};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);
        Ok(Client { connection: connection })
    }

    #[deprecated]
    pub async fn ping(&mut self) -> crate::Result<Frame> {
        let query = "PING".to_string();
        self.write(&Frame::String(query)).await?;
        let response = self.read().await?;
        Ok(response)
    }

    pub async fn run(&mut self, string: String) -> crate::Result<Frame> {
        self.write(&Frame::String(string)).await?;
        let response = self.read().await?;
        Ok(response)
    }

    async fn read(&mut self) -> crate::Result<Frame> {
        let response = self.connection.read().await?;

        match response {
            Some(result) => {
                Ok(result)
            },
            None => {
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");
                Err(err.into())
            }
        }
    }

    async fn write(&mut self, frame: &Frame) -> crate::Result<()> {
        let _ = self.connection.write(frame).await?;
        Ok(())
    }
}