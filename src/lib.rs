extern crate actix;
extern crate actix_web;
extern crate flexi_logger;
extern crate futures;
extern crate grpc;
// extern crate grpcio;
// extern crate grpcio_proto;
#[macro_use]
extern crate log;
extern crate protobuf;
extern crate tls_api;
extern crate tokio;

pub mod actor;
// pub mod grpc_rs;
pub mod hello;
pub mod hello_grpc;
pub mod server;

use std::error::Error;
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq)]
pub struct StringError(pub String);

impl StringError {
    pub fn new(s: &str) -> Self {
        StringError(String::from(s))
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

impl From<StringError> for actix_web::Error {
    fn from(err: StringError) -> actix_web::Error {
        actix_web::error::ErrorInternalServerError(err)
    }
}
