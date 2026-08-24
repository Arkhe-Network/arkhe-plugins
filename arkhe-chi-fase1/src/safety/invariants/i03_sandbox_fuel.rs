use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct SandboxFuelInvariant;

impl Invariant for SandboxFuelInvariant {
    fn id(&self) -> &'static str {
        "I-03"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Critical
    }

    fn check(&self, state: &SystemState) -> bool {
        state.sandbox_fuel >= state.config.min_fuel
    }

    fn margin(&self, state: &SystemState) -> f64 {
        if !self.check(state) {
            return 0.0;
        }
        // Fix R1: Use config.max_sandbox_fuel instead of hardcoded 1000.0
        let range = (state.config.max_sandbox_fuel - state.config.min_fuel) as f64;
        if range <= 0.0 {
            return 1.0;
        }
        let current = (state.sandbox_fuel - state.config.min_fuel) as f64;
        (current / range).clamp(0.0, 1.0)
    }
}
