pub mod trust_tier;
// Mock other required modules for level25.rs to compile
pub mod barrier {
    pub struct BarrierChecker;
    impl BarrierChecker {
        pub fn new() -> Self { Self }
        pub fn classify<T>(&self, _claim: &T) -> BarrierVerdict { BarrierVerdict::Pass }
    }
    pub enum BarrierVerdict {
        Pass,
        Barred { model: String, reason: String, confidence: f64 },
    }
}
pub mod ledger {
    use chrono::{DateTime, Utc};
    pub struct FailureLedger;
    impl FailureLedger {
        pub fn new() -> Self { Self }
        pub fn add(&self, _entry: FailureEntry) {}
    }
    #[derive(Debug, Clone)]
    pub enum DeflationClass {
        Novel,
        EquivalentToTarget,
        KnownTheoremRestated,
        Tautological,
    }
    pub struct FailureEntry {
        pub strategy_id: String,
        pub deflation_class: DeflationClass,
        pub kill_reason: String,
        pub validators: Vec<String>,
        pub timestamp: DateTime<Utc>,
        pub source_campaign: String,
    }
}
