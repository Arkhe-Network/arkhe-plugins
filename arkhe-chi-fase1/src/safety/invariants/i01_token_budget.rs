use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct TokenBudgetInvariant;

impl Invariant for TokenBudgetInvariant {
    fn id(&self) -> &'static str {
        "I-01"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Critical
    }

    fn check(&self, state: &SystemState) -> bool {
        state.token_budget >= 0 && state.token_budget <= state.config.max_tokens
    }

    fn margin(&self, state: &SystemState) -> f64 {
        if !self.check(state) {
            return 0.0;
        }
        let half = state.config.max_tokens as f64 / 2.0;
        let diff = (state.token_budget as f64 - half).abs();
        let normalized = diff / half; // 0 (center) to 1 (edge)
        (1.0 - normalized).clamp(0.0, 1.0)
    }
}
