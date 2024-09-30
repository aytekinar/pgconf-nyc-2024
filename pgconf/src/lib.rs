use std::{sync::LazyLock, time::Duration};

#[cfg(feature = "telemetry")]
use opentelemetry::{
    global,
    propagation::Injector,
    trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};

#[cfg(feature = "telemetry")]
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime};
use pgrx::prelude::*;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use vector_service_proto::ops::{
    vector_service_client::VectorServiceClient, DotProductRequest, VectorNormRequest,
};

::pgrx::pg_module_magic!();

#[pg_extern]
fn hello_pgconf() -> &'static str {
    "Hello, pgconf"
}

#[cfg(feature = "rand")]
#[pg_extern]
fn pgconf_gen_random() -> i32 {
    rand::random()
}

#[cfg(feature = "telemetry")]
struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

#[cfg(feature = "telemetry")]
impl<'a> Injector for MetadataMap<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        // .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
});

static CLIENT: LazyLock<Mutex<VectorServiceClient<Channel>>> = LazyLock::new(|| {
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:50051")
        .timeout(Duration::from_millis(1000));
    let client = RUNTIME
        .block_on(VectorServiceClient::connect(endpoint))
        .expect("could not connect");
    Mutex::new(client)
});

#[pg_extern]
fn vector_dot_product(v1: Vec<f32>, v2: Vec<f32>) -> f32 {
    RUNTIME.block_on(async {
        #[cfg(feature = "telemetry")]
        let meter = global::meter("vector-service")
            .u64_counter("dot_product")
            .init();

        #[cfg(feature = "telemetry")]
        meter.add(1, &[KeyValue::new("site", "client")]);

        #[cfg(feature = "telemetry")]
        let tracer = global::tracer("vector-service");

        #[cfg(feature = "telemetry")]
        let cx = Context::current_with_span(
            tracer
                .span_builder("dot_product/client")
                .with_kind(opentelemetry::trace::SpanKind::Client)
                .with_attributes(vec![KeyValue::new("component", "grpc")])
                .start(&tracer),
        );

        #[cfg(feature = "telemetry")]
        let mut request = tonic::Request::new(DotProductRequest {
            vector1: v1,
            vector2: v2,
        });

        #[cfg(not(feature = "telemetry"))]
        let request = tonic::Request::new(DotProductRequest {
            vector1: v1,
            vector2: v2,
        });

        #[cfg(feature = "telemetry")]
        cx.span().add_event(
            "request created",
            vec![
                KeyValue::new("vec1len", request.get_ref().vector1.len() as i64),
                KeyValue::new("vec2len", request.get_ref().vector2.len() as i64),
            ],
        );

        #[cfg(feature = "telemetry")]
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()));
        });

        let mut client = CLIENT.lock().await;
        let result = client
            .dot_product(request)
            .await
            .expect("could not get result")
            .into_inner()
            .result;

        #[cfg(feature = "telemetry")]
        cx.span().add_event(
            "response received",
            vec![KeyValue::new("result", result as f64)],
        );

        #[allow(clippy::let_and_return)]
        result
    })
}

#[pg_extern]
fn vector_norm(v: Vec<f32>) -> f32 {
    RUNTIME.block_on(async {
        #[cfg(feature = "telemetry")]
        let meter = global::meter("vector-service")
            .u64_counter("vector_norm")
            .init();

        #[cfg(feature = "telemetry")]
        meter.add(1, &[KeyValue::new("site", "client")]);

        #[cfg(feature = "telemetry")]
        let tracer = global::tracer("vector-service");

        #[cfg(feature = "telemetry")]
        let cx = Context::current_with_span(
            tracer
                .span_builder("vector_norm/client")
                .with_kind(opentelemetry::trace::SpanKind::Client)
                .with_attributes(vec![KeyValue::new("component", "grpc")])
                .start(&tracer),
        );

        #[cfg(feature = "telemetry")]
        let mut request = tonic::Request::new(VectorNormRequest { vector: v });

        #[cfg(not(feature = "telemetry"))]
        let request = tonic::Request::new(VectorNormRequest { vector: v });

        #[cfg(feature = "telemetry")]
        cx.span().add_event(
            "request created",
            vec![KeyValue::new(
                "veclen",
                request.get_ref().vector.len() as i64,
            )],
        );

        #[cfg(feature = "telemetry")]
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()));
        });

        let mut client = CLIENT.lock().await;
        let result = client
            .vector_norm(request)
            .await
            .expect("could not get result")
            .into_inner()
            .result;

        #[cfg(feature = "telemetry")]
        cx.span().add_event(
            "response received",
            vec![KeyValue::new("result", result as f64)],
        );

        #[allow(clippy::let_and_return)]
        result
    })
}

#[pg_guard]
pub extern "C" fn _PG_init() {
    // initialize the gRPC client
    let _ = CLIENT.blocking_lock();

    #[cfg(feature = "telemetry")]
    {
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
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_pgconf() {
        assert_eq!("Hello, pgconf", crate::hello_pgconf());
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
