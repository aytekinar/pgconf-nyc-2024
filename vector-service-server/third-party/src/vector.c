#include "vector.h"

__attribute__((target_clones("default", "avx", "avx2", "fma")))
float vector_dot_product(const float *a, const float *b, size_t n) {
    float sum = 0.0f;
    for (size_t i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}
