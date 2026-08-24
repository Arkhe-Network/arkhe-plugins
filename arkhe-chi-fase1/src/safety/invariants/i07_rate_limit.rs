use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct RateLimitInvariant;

impl Invariant for RateLimitInvariant {
    fn id(&self) -> &'static str {
        "I-07"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::High // Fix C1
    }

    fn check(&self, state: &SystemState) -> bool {
        state.rate_limit_remaining >= 0
    }

    fn margin(&self, state: &SystemState) -> f64 {
        if !self.check(state) {
            return 0.0;
        }
        let total = state.config.max_rate_limit as f64;
        if total <= 0.0 {
            return 1.0;
        }
        (state.rate_limit_remaining as f64 / total).clamp(0.0, 1.0)
    }
}
