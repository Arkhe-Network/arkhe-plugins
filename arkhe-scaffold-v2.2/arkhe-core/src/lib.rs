pub mod calibration {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ActivationRegime {
        ReLU,
        Sigmoid,
        Tanh,
        Hard,
    }

    pub fn activate(x: f64, regime: &ActivationRegime) -> f64 {
        match regime {
            ActivationRegime::ReLU => {
                if x > 0.0 {
                    x
                } else {
                    0.0
                }
            }
            ActivationRegime::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationRegime::Tanh => x.tanh(),
            ActivationRegime::Hard => {
                if x > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

pub mod symbolic {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SymbolicExpr {
        Claim(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SymbolicWitness {
        pub claim_id: String,
        pub evidence_text: String,
        pub source: String,
        pub confidence: f64,
        pub timestamp_us: u64,
    }

    pub trait SymbolicIRP {
        fn insert(&mut self, _id: &str, _witness: SymbolicWitness) {}
    }

    pub struct InMemoryIRP {}

    impl InMemoryIRP {
        pub fn new() -> Self {
            Self {}
        }
        pub fn insert(&mut self, _id: &str, _witness: SymbolicWitness) {}
    }

    impl Default for InMemoryIRP {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SymbolicIRP for InMemoryIRP {}
}
