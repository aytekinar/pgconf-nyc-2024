use std::{net::Ipv4Addr, sync::LazyLock};

use log::info;

use tokio::runtime::Runtime;
use vector_service_proto::ops::{
    vector_service_server::{VectorService, VectorServiceServer},
    DotProductRequest, DotProductResponse, VectorNormRequest, VectorNormResponse,
};

struct VectorServiceServerImpl;

// Implement the VectorService trait for VectorServiceServerImpl
#[tonic::async_trait]
impl VectorService for VectorServiceServerImpl {
    async fn dot_product(
        &self,
        request: tonic::Request<DotProductRequest>,
    ) -> Result<tonic::Response<DotProductResponse>, tonic::Status> {
        info!(target="server", metadata:?=request.metadata(); "request received" );

        let request = request.into_inner();

        let result = vector_service_server::vector_dot_product(&request.vector1, &request.vector2)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        info!(
            target = "server",
            vector1:? = request.vector1,
            vector2:? = request.vector2,
            result = result;
            "dot product computed"
        );

        Ok(tonic::Response::new(DotProductResponse { result }))
    }

    async fn vector_norm(
        &self,
        request: tonic::Request<VectorNormRequest>,
    ) -> Result<tonic::Response<VectorNormResponse>, tonic::Status> {
        info!(target="server", metadata:?=request.metadata(); "request received" );

        let request = request.into_inner();

        let result = vector_service_server::vector_norm(&request.vector)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        info!(
            target = "server",
            vector:? = request.vector,
            result = result;
            "vector_norm computed"
        );

        Ok(tonic::Response::new(VectorNormResponse { result }))
    }
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
});

fn main() {
    structured_logger::Builder::new().init();

    let service = VectorServiceServerImpl {};
    RUNTIME
        .block_on(
            tonic::transport::Server::builder()
                .add_service(VectorServiceServer::new(service))
                .serve(std::net::SocketAddrV4::new(Ipv4Addr::LOCALHOST, 50051).into()),
        )
        .expect("could not start server");
}
