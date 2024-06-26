#[cfg(feature = "rayon")]
use rayon::prelude::*;

use super::Embeddings;

/// Cosine distance between two vectors
///
/// When the features distances lengths don't match, the longer feature vector is truncated to
/// shorter one when the distance is calculated
///  
#[cfg(not(feature = "rayon"))]
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

#[cfg(feature = "rayon")]
#[inline]
pub fn cosine(vec_a: &Embeddings, vec_b: &Embeddings) -> f64 {
    assert_eq!(vec_a.len(), vec_b.len());

    let dot_product: f64 = vec_a
        .par_iter()
        .zip(vec_b.par_iter())
        .map(|(a, b)| a * b)
        .sum();
    let magnitude1: f64 = vec_a.par_iter().map(|a| a * a).sum::<f64>().sqrt();
    let magnitude2: f64 = vec_b.par_iter().map(|b| b * b).sum::<f64>().sqrt();

    1.0 - dot_product / (magnitude1 * magnitude2)
}
