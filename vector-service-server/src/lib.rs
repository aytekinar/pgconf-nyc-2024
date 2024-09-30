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
    if v1.len() != v2.len() {
        return Err(VectorServiceError {
            code: 1,
            message: "vectors must have the same length".to_string(),
        });
    }

    let result = unsafe {
        third_party::vector_dot_product(v1.as_ptr(), v2.as_ptr(), v1.len() as libc::size_t)
    };

    Ok(result)
}

fn sqrt(x: f32) -> Result<f32, VectorServiceError> {
    let result = x.sqrt();
    if result.is_nan() {
        Err(VectorServiceError {
            code: 2,
            message: "sqrt called with a strictly negative number".to_string(),
        })
    } else {
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
