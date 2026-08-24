use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct CapabilityInvariant;

impl Invariant for CapabilityInvariant {
    fn id(&self) -> &'static str {
        "I-08"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Critical
    }

    fn check(&self, state: &SystemState) -> bool {
        (state.model_capability & state.task_requirement) == state.task_requirement
    }

    fn margin(&self, state: &SystemState) -> f64 {
        // Fix R2: Reflect extra capacity correctly
        if !self.check(state) {
            return 0.0;
        }

        let required = state.task_requirement;
        let available = state.model_capability;

        let extra = available.count_ones() - required.count_ones();
        let max_extra = 64 - required.count_ones(); // Because it's u64 now

        if max_extra == 0 {
            1.0
        } else {
            (extra as f64 / max_extra as f64).clamp(0.0, 1.0)
        }
    }
}
