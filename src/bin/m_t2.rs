use std::sync::Arc;

use async_trait::async_trait;
use hyper::server::conn::Http;
use tokio::net::TcpListener;
use tokio::runtime::Builder;

use tokio::runtime::Runtime;
use tonic::Request;
use tonic::Response;
use tonic::Status;

use grpc_test::hello_server;
use grpc_test::HelloRequest;
use grpc_test::HelloResponse;

use mimalloc::MiMalloc;

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
    Builder::new_current_thread().enable_all().build().unwrap()
}


fn main() {
    let rt = tokio_rt();
    rt.block_on(async {
        let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();
        let listener = Arc::new(listener);
        let n: usize = std::env::var("GOMAXPROCS").unwrap().parse().unwrap();
        for _ in 0..n-1 {
            let listener = listener.clone();
            std::thread::spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(run_instance(listener));
            });
        }
        run_instance(listener).await;
    });
}


async fn run_instance(listener: Arc<TcpListener>) {
    let server = hello_server::HelloServer::new(HelloImpl {});

    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        let server = server.clone();
        tokio::spawn(async move {
            Http::new().serve_connection(stream, server).await.unwrap();
        });
    }
}
