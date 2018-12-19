//! An asynchronous RabbitMQ client for proxy engine
//!

use std::io;
use std::sync::Arc;

use amq_protocol::uri::AMQPUri;
use futures::future::Future;
use futures::IntoFuture;
use lapin_futures_rustls::lapin::channel::{Channel, ConfirmSelectOptions};
use lapin_futures_rustls::lapin::client::{Client, ConnectionOptions};
use log::error;
use tokio::executor::spawn;
use tokio_tcp::TcpStream;

use crate::rabbitmq::utils::get_address_to_rabbitmq;

/// Alias for the lapin client with TLS.
pub type LapinClient = Client<TcpStream>;
/// Alias for the lapin channel.
pub type LapinChannel = Channel<TcpStream>;

/// A future-based asynchronous RabbitMQ client.
pub struct RabbitMQClient {
    client: Arc<LapinClient>
}

impl RabbitMQClient {
    /// Initializes the inner fields of RabbitMQ client for future usage.
    pub fn connect(uri: &AMQPUri) -> impl Future<Item=Self, Error=io::Error> + Sync + Send + 'static {
        let address = get_address_to_rabbitmq(uri);
        let uri_inner = uri.clone();

        TcpStream::connect(&address)
            .and_then(|stream| Client::connect(stream, ConnectionOptions::from_uri(uri_inner)))
            .and_then(move |(client, heartbeat)| {
                spawn(heartbeat.map_err(|err| error!("Heartbeat error: {:?}", err)))
                    .into_future()
                    .map(|_| RabbitMQClient { client: Arc::new(client) })
                    .map_err(|error| {
                        error!("Occurred an error during spawning heartbeat future: {:?}", error);
                        io::Error::new(io::ErrorKind::Other, "Spawn error.")
                    })
            })
    }

    /// Returns a lapin channel as future, based on the lapin client instance.
    pub fn get_channel(&self) -> impl Future<Item=LapinChannel, Error=io::Error> + Sync + Send + 'static {
        let client = self.client.clone();
        client.create_confirm_channel(ConfirmSelectOptions::default())
    }
}