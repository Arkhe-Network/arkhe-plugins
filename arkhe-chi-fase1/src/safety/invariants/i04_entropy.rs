use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct EntropyInvariant;

impl Invariant for EntropyInvariant {
    fn id(&self) -> &'static str {
        "I-04"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Critical
    }

    fn check(&self, state: &SystemState) -> bool {
        state.entropy_bits >= state.config.min_entropy
    }

    fn margin(&self, state: &SystemState) -> f64 {
        // Fix D4: margin_entropy returns 0.0 for violations
        if !self.check(state) {
            return 0.0;
        }

        let extra = state.entropy_bits.saturating_sub(state.config.min_entropy) as f64;
        let baseline = state.config.min_entropy as f64;
        if baseline == 0.0 {
            1.0
        } else {
            (extra / baseline).clamp(0.0, 1.0)
        }
    }
}
