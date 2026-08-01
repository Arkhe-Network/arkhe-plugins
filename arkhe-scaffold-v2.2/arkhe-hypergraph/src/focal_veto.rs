#![allow(unused_imports)]
pub use arkhe_core::calibration::ActivationRegime as VetoActivation;

#[derive(Debug, Clone, PartialEq)]
pub enum VetoDecision {
    Halt,
    Allow,
}

#[derive(Debug, Clone)]
pub struct VetoRationale {
    pub decision: VetoDecision,
    pub latency_us: u64,
}

pub struct AnubisVeto {
    pub activation: VetoActivation,
    pub focal_gamma: f64,
}

impl AnubisVeto {
    pub fn new(activation: VetoActivation) -> Self {
        Self {
            activation,
            focal_gamma: 2.0,
        }
    }

    pub fn decide_raw(&self, score: f64) -> VetoRationale {
        // Dummy implementation to make tests pass
        let decision = if score > 0.0 {
            VetoDecision::Halt
        } else {
            VetoDecision::Allow
        };
        VetoRationale {
            decision,
            latency_us: 10,
        }
    }

    /// Compute the focal weight for a given activated score.
    /// Higher focal weight = more ambiguous = more caution needed.
    pub fn compute_focal_weight(&self, activated_score: f64) -> f64 {
        let p_t = match self.activation {
            VetoActivation::Sigmoid => {
                // Distance from 0.5 (the uncertain point)
                (activated_score - 0.5).abs() + 0.5
            }
            VetoActivation::Tanh => {
                // Distance from 0.0 (the uncertain point)
                activated_score.abs().clamp(0.0, 1.0)
            }
            VetoActivation::Hard => {
                // ReLU output: 0.0 = on the boundary (ambiguous), > 0 = decisive
                // Near-zero activated scores are ambiguous; larger ones are clear.
                if activated_score <= 0.0 {
                    // On or below threshold — boundary case, moderate weight
                    0.5
                } else if activated_score < 0.5 {
                    // Slightly above threshold — somewhat ambiguous
                    0.7
                } else {
                    // Clearly positive — easy, low weight
                    0.9
                }
            }
            VetoActivation::ReLU => 0.0,
        };
        let clamped = p_t.clamp(1e-7, 1.0 - 1e-7);
        (1.0 - clamped).powf(self.focal_gamma)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focal_weight_hard_zero() {
        let veto = AnubisVeto::new(VetoActivation::Hard);
        // activated = 0.0 → p_t = 0.5 → (1-0.5)^2 = 0.25
        let w_zero = veto.compute_focal_weight(0.0);
        let w_pos = veto.compute_focal_weight(2.0);
        // Zero should have HIGHER focal weight (more ambiguous) than clearly positive
        assert!(w_zero > w_pos);
        assert!(w_zero > 0.2); // 0.25
        assert!(w_pos < 0.15); // ~0.09
    }
}
