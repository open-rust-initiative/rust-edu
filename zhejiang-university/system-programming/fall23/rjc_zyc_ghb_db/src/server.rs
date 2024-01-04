use std::future::Future;
use std::path::Path;
use std::sync::{Arc};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{info, error, debug};

use crate::Frame;
use crate::error::Result;
use crate::sql::engine::Engine;
use crate::sql::engine::bitcask::KV;
use crate::sql::execution::ResultSet;
use crate::sql::session::Session;
use crate::storage::engine::bitcask::Bitcask;
use crate::{Connection, shutdown::Shutdown};

struct Listener<E: crate::storage::engine::Engine> {
    listener: TcpListener,
    db_holder: KV<E>,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

struct Handler<E: crate::storage::engine::Engine + 'static> {
    connection: Connection,
    session: Session<KV<E>>,
    shutdown: Shutdown,
    _shutdown_complete: mpsc::Sender<()>,
}

const MAX_CONNECTIONS: usize = 256;

impl<E: crate::storage::engine::Engine + 'static> Listener<E> {
    async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
                session: self.db_holder.session()?,
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                debug!("accept connection: {:?}", handler.connection);
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

impl<E: crate::storage::engine::Engine + 'static> Handler<E> {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_sql = tokio::select! {
                res = self.connection.read() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            if let Some(frame) = maybe_sql {
                debug!(?frame);
                if let Frame::String(string) = frame {
                    if string == "PING" {
                        debug!("[maybe_sql] {:?}", string);
                        self.connection.write(&Frame::String("PONG".to_string())).await?;
                    }
                    let response = tokio::task::block_in_place(|| {
                        let mut response = None;
                        let mut result_set = self.session.execute(&string);
                        match &mut result_set {
                            Ok(result_set) => {
                                match result_set {
                                    ResultSet::Query { columns, rows: ref mut resultrows } => {
                                        let schema = columns.iter().map(|c| c.name.clone().unwrap()).collect::<Vec<_>>();
                                        let schema = schema.join(" | ");

                                        let resultrows = std::mem::replace(resultrows, Box::new(std::iter::empty()));
                                        let rows = resultrows.map(|row| format!("{:?}", row.unwrap())).collect::<Vec<_>>().join("\n");
                                        response = Some(format!("{}\n{}", schema, rows));
                                    },
                                    other => {
                                        response = Some(format!("{:?}", other));
                                    }
                                }                                
                            },
                            Err(e) => {
                                response = Some(e.to_string());
                            }
                        };
                        response
                    });
                    if let Some(response) = response {
                        debug!(?response);
                        self.connection.write(&Frame::String(response)).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub async fn run(listener: TcpListener, shutdown: impl Future, data_path: &Path) -> Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let engine = Bitcask::new(data_path.to_path_buf())?;
    let db_gruad = KV::new(engine);

    let mut server = Listener {
        listener,
        db_holder: db_gruad,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }

    let Listener {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}