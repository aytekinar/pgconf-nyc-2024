use std::{net::Ipv4Addr, sync::LazyLock};

use log::info;

#[cfg(feature = "telemetry")]
use opentelemetry::{
    global,
    propagation::Extractor,
    trace::{get_active_span, mark_span_as_active, Span, Tracer},
    KeyValue,
};

#[cfg(feature = "telemetry")]
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime};

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

        #[cfg(feature = "telemetry")]
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&MetadataMap(request.metadata()))
        });

        #[cfg(feature = "telemetry")]
        let tracer = global::tracer("vector-service");

        #[cfg(feature = "telemetry")]
        let mut span = tracer
            .span_builder("dot_product/server")
            .with_kind(opentelemetry::trace::SpanKind::Server)
            .with_attributes(vec![KeyValue::new("component", "grpc")])
            .start_with_context(&tracer, &parent_cx);

        let request = request.into_inner();

        #[cfg(feature = "telemetry")]
        span.add_event(
            "request received",
            vec![
                KeyValue::new("vec1len", request.vector1.len() as i64),
                KeyValue::new("vec2len", request.vector2.len() as i64),
            ],
        );

        #[cfg(feature = "telemetry")]
        let meter = global::meter("vector-service")
            .u64_counter("dot_product")
            .init();

        #[cfg(feature = "telemetry")]
        meter.add(1, &[KeyValue::new("site", "server")]);

        #[cfg(feature = "telemetry")]
        let _active = mark_span_as_active(span);

        let result = vector_service_server::vector_dot_product(&request.vector1, &request.vector2)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        info!(
            target = "server",
            vector1:? = request.vector1,
            vector2:? = request.vector2,
            result = result;
            "dot product computed"
        );

        #[cfg(feature = "telemetry")]
        get_active_span(|span| {
            span.add_event(
                "result computed",
                vec![KeyValue::new("result", result as f64)],
            )
        });

        Ok(tonic::Response::new(DotProductResponse { result }))
    }

    async fn vector_norm(
        &self,
        request: tonic::Request<VectorNormRequest>,
    ) -> Result<tonic::Response<VectorNormResponse>, tonic::Status> {
        info!(target="server", metadata:?=request.metadata(); "request received" );

        #[cfg(feature = "telemetry")]
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&MetadataMap(request.metadata()))
        });

        #[cfg(feature = "telemetry")]
        let tracer = global::tracer("vector-service");

        #[cfg(feature = "telemetry")]
        let mut span = tracer
            .span_builder("vector_norm/server")
            .with_kind(opentelemetry::trace::SpanKind::Server)
            .with_attributes(vec![KeyValue::new("component", "grpc")])
            .start_with_context(&tracer, &parent_cx);

        let request = request.into_inner();

        #[cfg(feature = "telemetry")]
        span.add_event(
            "request received",
            vec![KeyValue::new("veclen", request.vector.len() as i64)],
        );

        #[cfg(feature = "telemetry")]
        let meter = global::meter("vector-service")
            .u64_counter("vector_norm")
            .init();

        #[cfg(feature = "telemetry")]
        meter.add(1, &[KeyValue::new("site", "server")]);

        #[cfg(feature = "telemetry")]
        let _active = mark_span_as_active(span);

        let result = vector_service_server::vector_norm(&request.vector)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;

        info!(
            target = "server",
            vector:? = request.vector,
            result = result;
            "vector_norm computed"
        );

        #[cfg(feature = "telemetry")]
        get_active_span(|span| {
            span.add_event(
                "result computed",
                vec![KeyValue::new("result", result as f64)],
            )
        });

        Ok(tonic::Response::new(VectorNormResponse { result }))
    }
}

#[cfg(feature = "telemetry")]
struct MetadataMap<'a>(&'a tonic::metadata::MetadataMap);

#[cfg(feature = "telemetry")]
impl<'a> Extractor for MetadataMap<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
});

#[cfg(feature = "telemetry")]
fn init_opentelemetry() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let meter_provider = RUNTIME.block_on(async {
        opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .build()
            .unwrap()
    });
    global::set_meter_provider(meter_provider);

    let tracer_provider = RUNTIME.block_on(async {
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(runtime::Tokio)
            .unwrap()
    });
    global::set_tracer_provider(tracer_provider);
}

fn main() {
    structured_logger::Builder::new().init();

    #[cfg(feature = "telemetry")]
    init_opentelemetry();

    let service = VectorServiceServerImpl {};
    RUNTIME
        .block_on(
            tonic::transport::Server::builder()
                .add_service(VectorServiceServer::new(service))
                .serve(std::net::SocketAddrV4::new(Ipv4Addr::LOCALHOST, 50051).into()),
        )
        .expect("could not start server");

    #[cfg(feature = "telemetry")]
    global::shutdown_tracer_provider();
}
