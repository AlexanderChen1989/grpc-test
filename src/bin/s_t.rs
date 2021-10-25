use async_trait::async_trait;
use tokio::runtime::Builder;
use tonic::transport::Server;

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

fn main() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async {
        let addr = "0.0.0.0:5000".parse().unwrap();
        let server = hello_server::HelloServer::new(HelloImpl {});
        
        Server::builder()
            .add_service(server)
            .serve(addr)
            .await
            .unwrap();
    });
}
