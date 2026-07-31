use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyClass {
    Inconsistent,
    Factual,
}

impl ConsistencyClass {
    pub fn is_risk(&self) -> bool {
        matches!(self, Self::Inconsistent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimOutcome {
    Vetoed,
    Allowed,
}
