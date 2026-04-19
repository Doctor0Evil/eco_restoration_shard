//! KER triad module - computes Knowledge (K), Eco-impact (E), and Risk-of-harm (R)

use serde::{Deserialize, Serialize};

/// KerTriad represents the three core metrics for ecosafety governance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KerTriad {
    pub k: f64,
    pub e: f64,
    pub r: f64,
}

/// Compute K (Knowledge factor) as the fraction of relevant coordinates with corridor-backed evidence.
/// 
/// K = n_corridor_backed / n_critical
pub fn compute_k(n_corridor_backed: usize, n_critical: usize) -> f64 {
    if n_critical == 0 {
        return 1.0; // No critical coords means fully knowledgeable by default
    }
    n_corridor_backed as f64 / n_critical as f64
}

/// Compute E (Eco-impact) from a benefit kernel normalized to [0,1].
/// 
/// E = (benefit_kernel - b_min) / (b_max - b_min)
pub fn compute_e(benefit_kernel: f64, b_min: f64, b_max: f64) -> f64 {
    if b_max <= b_min {
        return if benefit_kernel >= b_min { 1.0 } else { 0.0 };
    }
    let e = (benefit_kernel - b_min) / (b_max - b_min);
    e.clamp(0.0, 1.0)
}

/// Compute R (Risk-of-harm) as weighted sum of risk coordinate penetrations.
/// 
/// R = sum_i(w_i * r_i) / sum_i(w_i) where r_i is normalized coordinate penetration.
pub fn compute_r(coords: &[(String, f64)], weights: &HashMap<String, f64>) -> f64 {
    if coords.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (varid, value) in coords {
        let weight = weights.get(varid).copied().unwrap_or(1.0);
        weighted_sum += weight * value;
        weight_total += weight;
    }

    if weight_total <= 0.0 {
        return 0.0;
    }

    (weighted_sum / weight_total).clamp(0.0, 1.0)
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_k_full() {
        assert_eq!(compute_k(10, 10), 1.0);
        assert_eq!(compute_k(5, 10), 0.5);
        assert_eq!(compute_k(0, 0), 1.0);
    }

    #[test]
    fn test_compute_e_normalized() {
        assert_eq!(compute_e(50.0, 0.0, 100.0), 0.5);
        assert_eq!(compute_e(100.0, 0.0, 100.0), 1.0);
        assert_eq!(compute_e(0.0, 0.0, 100.0), 0.0);
    }

    #[test]
    fn test_compute_r_weighted() {
        let mut weights = HashMap::new();
        weights.insert("rtox".to_string(), 2.0);
        weights.insert("rmicro".to_string(), 1.0);

        let coords = vec![
            ("rtox".to_string(), 0.5),
            ("rmicro".to_string(), 1.0),
        ];

        let r = compute_r(&coords, &weights);
        // R = (2*0.5 + 1*1.0) / (2+1) = 2.0/3.0 ≈ 0.667
        assert!((r - 0.666666).abs() < 0.001);
    }
}
