use std::thread;
use async_trait::async_trait;
use tokio::net::TcpSocket;
use tokio::runtime::Builder;

use tokio::runtime::Runtime;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;

use grpc_test::hello_server;
use grpc_test::HelloRequest;
use grpc_test::HelloResponse;

use mimalloc::MiMalloc;
use tonic::transport::Server;

use affinity::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct HelloImpl {}

#[async_trait]
impl hello_server::Hello for HelloImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let reply = HelloResponse { greetings: request.into_inner().name };
        Ok(Response::new(reply))
    }
}

fn tokio_rt() -> Runtime {
    return Builder::new_current_thread().enable_all().build().unwrap();
}

fn main() {
    set_thread_affinity(&[0]).unwrap();
    
    let id = thread::current().id();
    println!("===> Thread {:?}", id);
    let n: usize = std::env::var("GOMAXPROCS").unwrap().parse().unwrap();
    for i in 0..n - 1 {
        thread::spawn(move || {
            set_thread_affinity(&[i+1]).unwrap();
            let id = thread::current().id();
            println!("===> Thread {:?}", id);
            let rt = tokio_rt();
            rt.block_on(run_server(i));
        });
    }
    let rt = tokio_rt();
    rt.block_on(run_server(n - 1));
}

async fn run_server(_i: usize) {
    // let addr = format!("0.0.0.0:500{}", i);
    let addr = "0.0.0.0:5000";
    println!("Listen on: {}", addr);
    let addr = addr.parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseport(true).unwrap();
    socket.bind(addr).unwrap();
    let listener = socket.listen(1024).unwrap();
    let stream = TcpListenerStream::new(listener);

    let server = hello_server::HelloServer::new(HelloImpl {});

    Server::builder()
        .add_service(server)
        .serve_with_incoming(stream)
        .await
        .unwrap();
}
