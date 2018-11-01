//! A reverse proxy application.
//!
//! This module provides a reverse proxy implementation for the Open
//! Matchmaking project.
//!
//! The purposes of the application  are handling client connections, applying
//! a middleware with checks for a token when it was specified and communicating
//! with a message broker for getting responses from microservices in the certain
//! format.
//!

use std::collections::{HashMap};
use std::error::{Error};
use std::net::{SocketAddr};
use std::sync::{Arc, Mutex};

use amq_protocol::uri::{AMQPUri};
use cli::{CliOptions};
use futures::sync::{mpsc};
use futures::{Future, Sink};
use futures::stream::{Stream};
use tokio::net::{TcpListener};
use tokio_tungstenite::{accept_async};
use tungstenite::protocol::{Message};

use engine::{Engine, MessageSender};
use error::{PathfinderError};
use rabbitmq::client::{RabbitMQClient};
use rabbitmq::utils::{get_uri};


/// A reverse proxy application.
pub struct Proxy {
    engine: Arc<Engine>,
    amqp_uri: Arc<AMQPUri>,
    connections: Arc<Mutex<HashMap<SocketAddr, MessageSender>>>
}


impl Proxy {
    /// Returns a new instance of a reverse proxy application.
    pub fn new(cli: &CliOptions) -> Proxy {
        let engine = Engine::new(cli);
        let amqp_uri = get_uri(cli);

        Proxy {
            engine: Arc::new(engine),
            amqp_uri: Arc::new(amqp_uri),
            connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /// Run the server on the specified address and port.
    pub fn run(&self, address: SocketAddr) {
        let listener = TcpListener::bind(&address).unwrap();
        info!("Listening on: {}", address);

        let engine = self.engine.clone();
        let connections = self.connections.clone();

        let server = |rabbitmq: Arc<RabbitMQClient>| {
            listener.incoming().for_each(move |stream| {
                let addr = stream.peer_addr().expect("Connected stream should have a peer address.");

                let engine_local = engine.clone();
                let rabbimq_local = rabbitmq.clone();
                let connections_local = connections.clone();

                accept_async(stream)
                    // Process the messages
                    .and_then(move |ws_stream| {
                        let connection_for_insert = connections_local.clone();
                        let connection_for_remove = connections_local.clone();
                        let connections_inner = connections_local.clone();

                        // Create a channel for the stream, which other sockets will use to
                        // send us messages. It could be used for broadcasting your data to
                        // another users in the future.
                        let (tx, rx) = mpsc::unbounded();
                        connection_for_insert.lock().unwrap().insert(addr, Arc::new(tx));

                        // Split the WebSocket stream so that it will be possible to work
                        // with the reading and writing halves separately.
                        let (sink, stream) = ws_stream.split();

                        // Read and process each message
                        let ws_reader = stream.for_each(move |message: Message| {

                            // Get references to required components
                            let addr_nested = addr.clone();
                            let connections_nested = connections_inner.clone();
                            let transmitter_nested = connections_nested.lock().unwrap()[&addr_nested].clone();
                            let rabbitmq_nested = rabbimq_local.clone();
                            let processing_request_future = engine_local.process_request(
                                message,
                                transmitter_nested,
                                rabbitmq_nested
                            );

                            tokio::spawn(processing_request_future);
                            Ok(())
                        });

                        // Write back prepared responses
                        let ws_writer = rx.fold(sink, |mut sink, msg| {
                            sink.start_send(msg).unwrap();
                            Ok(sink)
                        });

                        // Wait for either half to be done to tear down the other
                        let connection = ws_reader.map(|_| ()).map_err(|_| ())
                            .select(ws_writer.map(|_| ()).map_err(|_| ()));

                        // Close the connection after using
                        tokio::spawn(connection.then(move |_| {
                            connection_for_remove.lock().unwrap().remove(&addr);
                            debug!("Connection {} closed.", addr);
                            Ok(())
                        }));

                        Ok(())
                    })
                    // An error occurred during the WebSocket handshake
                    .or_else(|error| {
                        debug!("{}", error.description());
                        Ok(())
                    })
            })
        };

        // Run the server
        let server_future = self.get_rabbitmq_client()
            .map_err(|error| {
                error!("Lapin error: {:?}", error);
                ()
            })
            .and_then(|rabbitmq: Arc<RabbitMQClient>| {
                server(rabbitmq)
                    .map_err(|_error| ())
            });

        tokio::runtime::run(server_future);
    }

    fn get_rabbitmq_client(&self)
        -> impl Future<Item=Arc<RabbitMQClient>, Error=PathfinderError> + Sync + Send + 'static
    {
        let amqp_uri = self.amqp_uri.clone();
        RabbitMQClient::connect(amqp_uri.as_ref())
            .map(|client| Arc::new(client))
            .map_err(|error| {
                error!("Error in RabbitMQ client. Reason: {:?}", error);
                PathfinderError::Io(error)
            })
    }
}
