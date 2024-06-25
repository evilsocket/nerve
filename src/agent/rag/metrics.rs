use super::Embeddings;

/// Cosine distance between two vectors
///
/// When the features distances lengths don't match, the longer feature vector is truncated to
/// shorter one when the distance is calculated
///  
#[inline]
pub fn cosine(vec_a: &Embeddings, vec_b: &Embeddings) -> f64 {
    assert_eq!(vec_a.len(), vec_b.len());

    let mut a_dot_b: f64 = 0.0;
    let mut a_mag: f64 = 0.0;
    let mut b_mag: f64 = 0.0;

    let vec_size = vec_a.len();

    for i in 0..vec_size {
        a_dot_b += vec_a[i] * vec_b[i];
        a_mag += vec_a[i] * vec_a[i];
        b_mag += vec_b[i] * vec_b[i];
    }

    1.0 - (a_dot_b / (a_mag.sqrt() * b_mag.sqrt()))
}
