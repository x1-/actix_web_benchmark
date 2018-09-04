use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};
use std::time::Duration;

use actix::prelude::*;
use actix_web::{error, server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse};
use actix_web::http::Method;
use actix_web::middleware::{Logger as ActixLogger};
use flexi_logger::{detailed_format, Logger};
use futures::Future;
use futures::future;
use grpc;
// use grpcio::{CallOption, ChannelBuilder, EnvBuilder};
use tokio;

use actor::*;
use hello::*;
use hello_grpc::*;

struct ApplicationState {
    counter     : AtomicUsize,
    echo_addr   : Addr<EchoActor>,
    grpc_client : HelloServiceClient,
}

pub struct Server;
impl Server {

    ///
    /// HTTPサーバを開始します.
    ///
    pub fn start(&self) {

        Logger::with_env_or_str("info")
            .format(detailed_format)
            .print_message()
            .append()
            .start()
            .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

        let sys = actix::System::new("BenchServer");

        let echo_addr: Addr<EchoActor> =
            EchoActor::new("Test")
                      .start();


        ///// grpc-rs
        // let env = Arc::new(EnvBuilder::new().build());
        // let ch = ChannelBuilder::new(env).connect("xxx.xxx.xxx.xxx:50051");
        // let client = HelloServiceClient::new(ch);

        ///// grpc-rust
        let mut client_conf = grpc::ClientConf::new();
        client_conf.http.connection_timeout = Some(Duration::from_millis(200));
        let client = HelloServiceClient::new_plain("xxx.xxx.xxx.xxx", 50051, client_conf).unwrap();


        let state = Arc::new(ApplicationState {
            counter     : AtomicUsize::new(0),
            echo_addr   : echo_addr.clone(),
            grpc_client : client,
        });

        server::HttpServer::new(move || {
            vec![
                App::with_state(state.clone())
                    .middleware(ActixLogger::default())
                    .resource("/health",      |r| r.method(Method::GET).f(Server::health))
                    .resource("/sleep",       |r| r.method(Method::GET).f(Server::sleep))
                    .resource("/count",       |r| r.method(Method::GET).f(Server::count))
                    .resource("/s_future",    |r| r.method(Method::GET).f(Server::sleep_and_future))
                    .resource("/f_sleep",     |r| r.method(Method::GET).f(Server::future_sleep))
                    .resource("/send_wait",   |r| r.method(Method::GET).f(Server::send_wait))
                    .resource("/fsend_wait",  |r| r.method(Method::GET).f(Server::future_send_wait))
                    .resource("/send_nowait", |r| r.method(Method::GET).f(Server::send_no_wait))
                    .resource("/grpc_wait",   |r| r.method(Method::GET).f(Server::grpc_wait))
                    .resource("/grpc_per",    |r| r.method(Method::GET).f(Server::grpc_per))
                    .resource("/fgrpc_wait",  |r| r.method(Method::GET).a(Server::future_grpc_wait))
                    // .resource("/fgrpc_wait_a",|r| r.method(Method::GET).a(Server::future_grpc_wait_async))
            ]
        })
            .backlog(7168)
            .keep_alive(server::KeepAlive::Timeout(0))
            .shutdown_timeout(60)
            .workers(8096)
            .bind("0.0.0.0:8888")
            .unwrap()
            .start();

        let _ = sys.run();
    }

    ///
    /// ヘルスチェック用エントリポイントです.
    /// 
    fn health(_: &HttpRequest<Arc<ApplicationState>>) -> &'static str { "ok" }

    ///
    /// sleep して スレッドをブロッキングします.
    /// 
    fn sleep(_: &HttpRequest<Arc<ApplicationState>>) -> &'static str {
        thread::sleep(time::Duration::from_millis(100));
        "done"
    }

    ///
    /// sleep してから、Future を返します.
    /// 
    fn sleep_and_future(_: &HttpRequest<Arc<ApplicationState>>) -> FutureResponse<&'static str> {
        thread::sleep(time::Duration::from_millis(100));
        Box::new(
            future::ok("s_future done")
        ).responder()
    }

    ///
    /// Future 内で　sleep します.
    /// 
    fn future_sleep(_: &HttpRequest<Arc<ApplicationState>>) -> FutureResponse<&'static str> {
        Box::new(
            Server::fsleep()
        ).responder()
    }

    fn fsleep() -> impl Future<Item = &'static str, Error = error::Error> {
        thread::sleep(time::Duration::from_millis(100));
        future::ok("fsleep done")
    }

    ///
    /// State の AtomicUsize をインクリメントします.
    /// 
    fn count(req: &HttpRequest<Arc<ApplicationState>>) -> String {
        let n = Server::fetch_increment(&req.state().counter);
        format!("count: {}", n)
    }

    ///
    /// 引数 `n` をインクリメントして返却します.
    /// `n` が `usize::max_value` に到達した場合は `0` に戻します.
    /// 
    /// @n: アトミックな符号なし整数
    /// 
    fn fetch_increment(n: &AtomicUsize) -> usize {
        let count = n.fetch_add(1, Ordering::SeqCst) + 1;
        if count == usize::max_value() {
            n.compare_and_swap(usize::max_value(), 0, Ordering::SeqCst);
        }
        count
    }

    ///
    /// Actor からの戻りを待ち受けます.
    /// 
    fn send_wait(req: &HttpRequest<Arc<ApplicationState>>) -> String {
        let res = req.state().echo_addr.send(Greeting {message: String::from("send_wait")});
        let s = res.wait();
        format!("{:?}", s)
    }

    ///
    /// Future 内で Actor からの戻りを待ち受けます.
    /// 
    fn future_send_wait(req: &HttpRequest<Arc<ApplicationState>>) -> FutureResponse<String> {
        Box::new(
            Server::fsend_wait(req.state().echo_addr.clone())
        ).responder()
    }

    fn fsend_wait(addr: Addr<EchoActor>) -> impl Future<Item = String, Error = error::Error> {
        addr.send(Greeting {message: String::from("fsend_wait")})
            .map(|s| format!("{:?}", s))
            .map_err(|e| error::ErrorInternalServerError(e))
    }

    ///
    /// Actor からの戻りを非同期で待ち受けます.
    /// 
    fn send_no_wait(req: &HttpRequest<Arc<ApplicationState>>) -> &'static str {
        let res = req.state().echo_addr.send(Greeting {message: String::from("send_wait")});
        tokio::spawn(res.map(|_| {
            debug!("echo_addr.send(Greeting)")
        }).map_err(|e|error!("{:?}", e)));
        "send and no wait"
    }

    ///////// stepancheg/grpc-rust 版
    ///
    /// gRPC通信を同期的に行います.
    /// 
    fn grpc_wait(req: &HttpRequest<Arc<ApplicationState>>) -> String {
        let mut v = HelloRequest::new();
        v.set_id(String::from("000001"));
        v.set_message(String::from("rustcean loves rust"));

        let resp = req.state().grpc_client.say(grpc::RequestOptions::new(), v);

        format!("grpc: {:?}", resp.wait())
    }
    ///
    /// gRPCクライアントを都度生成します.
    /// 
    fn grpc_per(_: &HttpRequest<Arc<ApplicationState>>) -> String {

        let mut client_conf = grpc::ClientConf::new();
        client_conf.http.connection_timeout = Some(Duration::from_millis(200));
        let client = HelloServiceClient::new_plain("xxx.xxx.xxx.xxx", 50001, client_conf).unwrap();

        let mut v = HelloRequest::new();
        v.set_id(String::from("000001"));
        v.set_message(String::from("rustcean loves rust"));

        let resp = client.say(grpc::RequestOptions::new(), v);

        format!("grpc: {:?}", resp.wait())
    }

    ///
    /// Future 内で gRPC通信 の戻りを待ち受けます.
    /// 
    fn future_grpc_wait(req: &HttpRequest<Arc<ApplicationState>>) -> impl Future<Item = HttpResponse, Error = error::Error> {
        Server::fgrpc_wait(&req.state().grpc_client).and_then(|s| {
            Ok(HttpResponse::Ok()
                .content_type("plain/text")
                .body(s)
                .into())
        })
    }

    fn fgrpc_wait(client: &HelloServiceClient) -> impl Future<Item = String, Error = error::Error> {
        let mut v = HelloRequest::new();
        v.set_id(String::from("000001"));
        v.set_message(String::from("rustcean loves rust"));

        client.say(grpc::RequestOptions::new(), v)
            .drop_metadata()
            .map_err(|e| error::ErrorInternalServerError(e))
            .and_then(|s| {
                future::ok(format!("grpc: {:?}", s))
            })
    }

    ///////// pingcap/grpc-rs 版
    // ///
    // /// gRPC通信を同期的に行います.
    // /// 
    // fn grpc_wait(req: &HttpRequest<Arc<ApplicationState>>) -> String { 
 
    //     let mut v = HelloRequest::new();
    //     v.set_id(String::from("000001"));
    //     v.set_message(String::from("rustcean loves rust"));

    //     let opt = CallOption::default().timeout(Duration::from_millis(500));
    //     let resp = req.state().grpc_client.say_opt(&v, opt);
    //     format!("grpc: {:?}", resp)
    // }

    // ///
    // /// Future 内で gRPC通信 の戻りを待ち受けます.
    // /// 
    // fn future_grpc_wait(req: &HttpRequest<Arc<ApplicationState>>) -> FutureResponse<HttpResponse> {
    //     Box::new(
    //         Server::fgrpc_wait(&req.state().grpc_client).and_then(|s| {
    //             Ok(HttpResponse::Ok()
    //                 .content_type("plain/text")
    //                 .body(s)
    //                 .into())
    //         })
    //     ).responder()
    // }

    // fn fgrpc_wait(client: &HelloServiceClient) -> impl Future<Item = String, Error = error::Error> {

    //     let mut v = HelloRequest::new();
    //     v.set_id(String::from("000001"));
    //     v.set_message(String::from("rustcean loves rust"));

    //     let opt = CallOption::default().timeout(Duration::from_millis(500));
    //     let resp = client.say_async_opt(&v, opt).unwrap();
    //     resp.and_then(|s| { future::ok(format!("grpc: {:?}", s)) })
    //         .map_err(|e| error::ErrorInternalServerError(e))
    // }

    // ///
    // /// Future 内で gRPC通信 の戻りを待ち受けます.
    // /// 
    // fn future_grpc_wait_async(req: &HttpRequest<Arc<ApplicationState>>) -> impl Future<Item = HttpResponse, Error = error::Error> {
    //     Server::fgrpc_wait_async(&req.state().grpc_client).and_then(|s| {
    //         Ok(HttpResponse::Ok()
    //             .content_type("plain/text")
    //             .body(s)
    //             .into())
    //     })
    // }

    // fn fgrpc_wait_async(client: &HelloServiceClient) -> impl Future<Item = String, Error = error::Error> {

    //     let mut v = HelloRequest::new();
    //     v.set_id(String::from("000001"));
    //     v.set_message(String::from("rustcean loves rust"));

    //     let opt = CallOption::default().timeout(Duration::from_millis(500));
    //     let resp = client.say_async_opt(&v, opt).unwrap();
    //     resp.and_then(|s| { future::ok(format!("grpc: {:?}", s)) })
    //         .map_err(|e| error::ErrorInternalServerError(e))
    // }
}
