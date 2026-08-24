use crate::safety::symmetry_generator::{Invariant, InvariantClass, SystemState};

pub struct SignatureInvariant;

impl Invariant for SignatureInvariant {
    fn id(&self) -> &'static str {
        "I-06"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Critical
    }

    fn check(&self, state: &SystemState) -> bool {
        state.signature_valid
    }
}
