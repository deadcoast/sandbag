//! Information-theoretic utilities (entropy, compression metrics)

/// Compute binary entropy H(p) in bits, where p in [0,1].
/// Returns 0.0 for p=0 or p=1.
pub fn binary_entropy(probability_true: f64) -> f64 {
    let p = probability_true.clamp(0.0, 1.0);
    if (p - 0.0).abs() < f64::EPSILON || (p - 1.0).abs() < f64::EPSILON {
        0.0
    } else {
        -p * (p.log2()) - (1.0 - p) * ((1.0 - p).log2())
    }
}

/// Shannon entropy of a discrete distribution in bits.
/// `probs` should sum to ~1.0; zero probabilities are ignored.
pub fn shannon_entropy(probs: &[f64]) -> f64 {
    let mut entropy = 0.0;
    for &p in probs {
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Information content I(x) = -log2 p(x). For p <= 0 returns `f64::INFINITY`.
pub fn information_content(p: f64) -> f64 {
    if p <= 0.0 {
        f64::INFINITY
    } else {
        -p.log2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_entropy_bounds() {
        assert!((binary_entropy(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((binary_entropy(1.0) - 0.0).abs() < f64::EPSILON);
        let mid = binary_entropy(0.5);
        assert!(mid > 0.9 && mid < 1.1);
    }

    #[test]
    fn test_shannon_entropy() {
        let h = shannon_entropy(&[0.5, 0.5]);
        assert!((h - 1.0).abs() < 1e-9);
    }
}
