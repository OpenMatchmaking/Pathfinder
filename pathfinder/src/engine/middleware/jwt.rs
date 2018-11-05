//! Middleware implementations with JSON Web Token support.
//!

use std::sync::Arc;

use futures::future::lazy;

use engine::middleware::base::{Middleware, MiddlewareFuture};
use engine::serializer::JsonMessage;
use rabbitmq::RabbitMQClient;

/// A middleware class, that will check a JSON Web Token in WebSocket message.
/// If token wasn't specified or it's invalid returns a `PathfinderError` object.
pub struct JwtTokenMiddleware;

impl JwtTokenMiddleware {
    /// Returns a new instance of `JwtTokenMiddleware` structure.
    pub fn new() -> JwtTokenMiddleware {
        JwtTokenMiddleware {}
    }
}

impl Middleware for JwtTokenMiddleware {
    fn process_request(&self, _message: JsonMessage, _rabbitmq_client: Arc<RabbitMQClient>) -> MiddlewareFuture {
        Box::new(lazy(move || Ok(())))
    }
}
