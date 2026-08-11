pub mod trust_tier;
pub mod barrier {
    #[derive(Debug, Clone)]
    pub struct BarrierChecker;
    impl Default for BarrierChecker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BarrierChecker {
        pub fn new() -> Self {
            Self
        }
        pub fn classify(&self, _claim: &crate::trust_tier::TheoremClaim) -> BarrierVerdict {
            BarrierVerdict::Pass
        }
    }
    #[derive(Debug, Clone)]
    pub enum BarrierVerdict {
        Pass,
        Barred {
            model: String,
            reason: String,
            confidence: f64,
        },
    }
}
pub mod ledger {
    use chrono::{DateTime, Utc};
    #[derive(Debug, Clone)]
    pub struct FailureLedger;
    impl Default for FailureLedger {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FailureLedger {
        pub fn new() -> Self {
            Self
        }
        pub fn add(&mut self, _entry: FailureEntry) -> Result<(), ()> {
            Ok(())
        }
    }
    #[derive(Debug, Clone)]
    pub enum DeflationClass {
        EquivalentToTarget,
        KnownTheoremRestated,
        Tautological,
        Novel,
    }
    #[derive(Debug, Clone)]
    pub struct FailureEntry {
        pub strategy_id: String,
        pub deflation_class: DeflationClass,
        pub kill_reason: String,
        pub validators: Vec<String>,
        pub timestamp: DateTime<Utc>,
        pub source_campaign: String,
    }
}
