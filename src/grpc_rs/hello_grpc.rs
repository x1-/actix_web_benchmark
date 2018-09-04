// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_HELLO_SERVICE_SAY: ::grpcio::Method<super::hello::HelloRequest, super::hello::HelloResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/HelloService/Say",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct HelloServiceClient {
    client: ::grpcio::Client,
}

impl HelloServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        HelloServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn say_opt(&self, req: &super::hello::HelloRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::hello::HelloResponse> {
        self.client.unary_call(&METHOD_HELLO_SERVICE_SAY, req, opt)
    }

    pub fn say(&self, req: &super::hello::HelloRequest) -> ::grpcio::Result<super::hello::HelloResponse> {
        self.say_opt(req, ::grpcio::CallOption::default())
    }

    pub fn say_async_opt(&self, req: &super::hello::HelloRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::hello::HelloResponse>> {
        self.client.unary_call_async(&METHOD_HELLO_SERVICE_SAY, req, opt)
    }

    pub fn say_async(&self, req: &super::hello::HelloRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::hello::HelloResponse>> {
        self.say_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait HelloService {
    fn say(&self, ctx: ::grpcio::RpcContext, req: super::hello::HelloRequest, sink: ::grpcio::UnarySink<super::hello::HelloResponse>);
}

pub fn create_hello_service<S: HelloService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_HELLO_SERVICE_SAY, move |ctx, req, resp| {
        instance.say(ctx, req, resp)
    });
    builder.build()
}
