#[cfg(feature = "telemetry")]
use opentelemetry::{
    global,
    trace::{Span, SpanKind, Tracer},
    KeyValue,
};

pub mod third_party {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug)]
pub struct VectorServiceError {
    code: i32,
    message: String,
}

impl std::fmt::Display for VectorServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "VectorServiceError: code={}, message={}",
            self.code, self.message
        )
    }
}

impl std::error::Error for VectorServiceError {}

pub fn vector_dot_product(v1: &[f32], v2: &[f32]) -> Result<f32, VectorServiceError> {
    #[cfg(feature = "telemetry")]
    let tracer = global::tracer("vector-service");

    #[cfg(feature = "telemetry")]
    let mut span = tracer
        .span_builder("dot_product/impl")
        .with_kind(SpanKind::Internal)
        .start(&tracer);

    #[cfg(feature = "telemetry")]
    span.add_event(
        "dot_product/impl called",
        vec![
            KeyValue::new("v1len", v1.len() as i64),
            KeyValue::new("v2len", v2.len() as i64),
        ],
    );

    if v1.len() != v2.len() {
        return Err(VectorServiceError {
            code: 1,
            message: "vectors must have the same length".to_string(),
        });
    }

    let result = unsafe {
        third_party::vector_dot_product(v1.as_ptr(), v2.as_ptr(), v1.len() as libc::size_t)
    };

    #[cfg(feature = "telemetry")]
    span.add_event(
        "result computed",
        vec![KeyValue::new("result", result as f64)],
    );

    Ok(result)
}

fn sqrt(x: f32) -> Result<f32, VectorServiceError> {
    #[cfg(feature = "telemetry")]
    let tracer = global::tracer("vector-service");

    #[cfg(feature = "telemetry")]
    let mut span = tracer
        .span_builder("sqrt/impl")
        .with_kind(SpanKind::Internal)
        .start(&tracer);

    #[cfg(feature = "telemetry")]
    span.add_event("sqrt/impl called", vec![KeyValue::new("x", x as f64)]);

    let result = x.sqrt();
    if result.is_nan() {
        Err(VectorServiceError {
            code: 2,
            message: "sqrt called with a strictly negative number".to_string(),
        })
    } else {
        #[cfg(feature = "telemetry")]
        span.add_event(
            "result computed",
            vec![KeyValue::new("result", result as f64)],
        );
        Ok(result)
    }
}

pub fn vector_dot_product_inefficient(v1: Vec<f32>, v2: Vec<f32>) -> f32 {
    let mut result = 0.0;
    for i in 0..v1.len() {
        result += v1[i] * v2[i];
    }
    result
}

pub fn vector_dot_product_efficient(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
}

pub fn vector_norm(v: &[f32]) -> Result<f32, VectorServiceError> {
    vector_dot_product(v, v).and_then(sqrt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_dot_product_success() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![4.0, 5.0, 6.0];
        assert_eq!(32.0, vector_dot_product(&v1, &v2).unwrap());
    }

    #[test]
    fn test_vector_dot_product_error() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![4.0, 5.0];
        assert_eq!(
            "VectorServiceError: code=1, message=vectors must have the same length",
            format!("{}", vector_dot_product(&v1, &v2).unwrap_err())
        );
    }
}
