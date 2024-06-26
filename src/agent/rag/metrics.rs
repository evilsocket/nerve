use rayon::prelude::*;

use super::Embeddings;

/// Cosine distance between two vectors
///
/// When the features distances lengths don't match, the longer feature vector is truncated to
/// shorter one when the distance is calculated
///  
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
