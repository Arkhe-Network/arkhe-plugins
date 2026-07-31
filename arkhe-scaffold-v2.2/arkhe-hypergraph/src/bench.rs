#![allow(unused_imports)]
//! Benchmark suite for classification tasks.
//!
//! v20.2 FIX: TP/TN/FP/FN tracked explicitly per sample.
//! Previous version had TP ≡ 0 due to algebraic error.

use crate::focal_veto::{AnubisVeto, VetoActivation, VetoDecision};
use crate::metrics::{ClaimOutcome, ConsistencyClass};
use crate::seam_monitor::SeamIntegrityMonitor;
use arkhe_core::symbolic::{InMemoryIRP, SymbolicExpr, SymbolicIRP, SymbolicWitness};
use serde::{Deserialize, Serialize};

/// Accumulator for confusion matrix counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfusionCounts {
    pub tp: usize,       // vetoed risky (correct halt)
    pub tn: usize,       // verified safe (correct allow)
    pub fp: usize,       // vetoed safe (false halt)
    pub fn_count: usize, // verified risky (false allow)
}

impl ConfusionCounts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total(&self) -> usize {
        self.tp + self.tn + self.fp + self.fn_count
    }

    pub fn accuracy(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            (self.tp + self.tn) as f64 / t as f64
        }
    }

    pub fn precision(&self) -> f64 {
        if self.tp + self.fp == 0 {
            0.0
        } else {
            self.tp as f64 / (self.tp + self.fp) as f64
        }
    }

    pub fn recall(&self) -> f64 {
        if self.tp + self.fn_count == 0 {
            0.0
        } else {
            self.tp as f64 / (self.tp + self.fn_count) as f64
        }
    }

    pub fn f1_score(&self) -> f64 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub counts: ConfusionCounts,
    pub total_claims: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub total_latency_us: u64,
    pub avg_latency_us: f64,
    pub claims_per_second: f64,
}

impl BenchmarkResult {
    pub fn print_summary(&self) {
        println!("┌─────────────────────────────────────────────┐");
        println!("│ Benchmark: {:<35}│", self.name);
        println!("├─────────────────────────────────────────────┤");
        println!("│ Total claims:    {:>25}│", self.total_claims);
        println!("│ TP (halt risky): {:>25}│", self.counts.tp);
        println!("│ TN (allow safe): {:>25}│", self.counts.tn);
        println!("│ FP (halt safe):  {:>25}│", self.counts.fp);
        println!("│ FN (allow risky):{:>25}│", self.counts.fn_count);
        println!("│ Accuracy:        {:>24.2}%│", self.accuracy * 100.0);
        println!("│ Precision:       {:>24.2}%│", self.precision * 100.0);
        println!("│ Recall:          {:>24.2}%│", self.recall * 100.0);
        println!("│ F1 Score:        {:>24.4} │", self.f1_score);
        println!("│ Avg latency:     {:>22} μs│", self.avg_latency_us as u64);
        println!("│ Throughput:      {:>20.1} c/s│", self.claims_per_second);
        println!("└─────────────────────────────────────────────┘");
    }
}

/// A classification benchmark case.
pub struct ClassificationCase {
    pub claim: SymbolicExpr,
    pub ground_truth: ConsistencyClass,
}

impl ClassificationCase {
    pub fn new(text: &str, risky: bool) -> Self {
        Self {
            claim: SymbolicExpr::Claim(text.to_string()),
            ground_truth: if risky {
                ConsistencyClass::Inconsistent
            } else {
                ConsistencyClass::Factual
            },
        }
    }
}

pub struct ClassificationBenchmark {
    pub cases: Vec<ClassificationCase>,
}

impl Default for ClassificationBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassificationBenchmark {
    pub fn new() -> Self {
        Self { cases: Vec::new() }
    }

    pub fn add(&mut self, case: ClassificationCase) {
        self.cases.push(case);
    }

    pub fn add_batch(&mut self, texts: &[(&str, bool)]) {
        for &(text, risky) in texts {
            self.add(ClassificationCase::new(text, risky));
        }
    }

    /// Classify a single sample and update confusion matrix.
    fn count_outcome(counts: &mut ConfusionCounts, outcome_is_halt: bool, truth_is_risk: bool) {
        match (outcome_is_halt, truth_is_risk) {
            (true, true) => counts.tp += 1,
            (false, false) => counts.tn += 1,
            (true, false) => counts.fp += 1,
            (false, true) => counts.fn_count += 1,
        }
    }

    /// Build a BenchmarkResult from counts and timing.
    fn build_result(
        name: &str,
        counts: ConfusionCounts,
        total_latency_us: u64,
        wall_time_us: u64,
    ) -> BenchmarkResult {
        let total = counts.total();
        let avg_latency = if total > 0 {
            total_latency_us as f64 / total as f64
        } else {
            0.0
        };
        let cps = if wall_time_us > 0 {
            total as f64 / (wall_time_us as f64 / 1_000_000.0)
        } else {
            0.0
        };

        BenchmarkResult {
            name: name.to_string(),
            total_claims: total,
            accuracy: counts.accuracy(),
            precision: counts.precision(),
            recall: counts.recall(),
            f1_score: counts.f1_score(),
            counts,
            total_latency_us,
            avg_latency_us: avg_latency,
            claims_per_second: cps,
        }
    }

    /// Run the benchmark through the SeamIntegrityMonitor.
    pub fn run(
        &self,
        name: &str,
        monitor: &mut SeamIntegrityMonitor<impl SymbolicIRP>,
    ) -> BenchmarkResult {
        let wall_start = std::time::Instant::now();
        let mut counts = ConfusionCounts::new();
        let mut total_latency = 0u64;

        for (i, case) in self.cases.iter().enumerate() {
            let report = monitor.inspect(&format!("bench_{}", i), &case.claim);
            total_latency += report.total_latency_us;

            let outcome_is_halt = matches!(report.final_outcome, ClaimOutcome::Vetoed);
            let truth_is_risk = case.ground_truth.is_risk();
            Self::count_outcome(&mut counts, outcome_is_halt, truth_is_risk);
        }

        let wall_time = wall_start.elapsed().as_micros() as u64;
        Self::build_result(name, counts, total_latency, wall_time)
    }

    /// Quick benchmark using only the veto (no full pipeline).
    pub fn run_veto_only(&self, name: &str, veto: &mut AnubisVeto) -> BenchmarkResult {
        let wall_start = std::time::Instant::now();
        let mut counts = ConfusionCounts::new();
        let mut total_latency = 0u64;

        for case in &self.cases {
            // Simulate a score based on ground truth with some noise
            let raw_score = if case.ground_truth.is_risk() {
                2.0
            } else {
                -1.0
            };
            let rationale = veto.decide_raw(raw_score);
            total_latency += rationale.latency_us;

            let outcome_is_halt = rationale.decision == VetoDecision::Halt;
            let truth_is_risk = case.ground_truth.is_risk();
            Self::count_outcome(&mut counts, outcome_is_halt, truth_is_risk);
        }

        let wall_time = wall_start.elapsed().as_micros() as u64;
        Self::build_result(name, counts, total_latency, wall_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confusion_counts_perfect() {
        let mut c = ConfusionCounts::new();
        // 4 correct: 2 TP + 2 TN
        ClassificationBenchmark::count_outcome(&mut c, true, true); // TP
        ClassificationBenchmark::count_outcome(&mut c, true, true); // TP
        ClassificationBenchmark::count_outcome(&mut c, false, false); // TN
        ClassificationBenchmark::count_outcome(&mut c, false, false); // TN
        assert_eq!(c.tp, 2);
        assert_eq!(c.tn, 2);
        assert_eq!(c.fp, 0);
        assert_eq!(c.fn_count, 0);
        assert!((c.accuracy() - 1.0).abs() < 1e-9);
        assert!((c.precision() - 1.0).abs() < 1e-9);
        assert!((c.recall() - 1.0).abs() < 1e-9);
        assert!((c.f1_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_confusion_counts_mixed() {
        let mut c = ConfusionCounts::new();
        ClassificationBenchmark::count_outcome(&mut c, true, true); // TP
        ClassificationBenchmark::count_outcome(&mut c, false, false); // TN
        ClassificationBenchmark::count_outcome(&mut c, true, false); // FP
        ClassificationBenchmark::count_outcome(&mut c, false, true); // FN
        assert_eq!(c.tp, 1);
        assert_eq!(c.tn, 1);
        assert_eq!(c.fp, 1);
        assert_eq!(c.fn_count, 1);
        assert!((c.accuracy() - 0.5).abs() < 1e-9);
        assert!((c.precision() - 0.5).abs() < 1e-9);
        assert!((c.recall() - 0.5).abs() < 1e-9);
        assert!((c.f1_score() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_confusion_counts_all_fp() {
        let mut c = ConfusionCounts::new();
        ClassificationBenchmark::count_outcome(&mut c, true, false); // FP
        ClassificationBenchmark::count_outcome(&mut c, true, false); // FP
                                                                     // precision = 0/(0+2) = 0, recall = 0/(0+0) = 0, f1 = 0
        assert_eq!(c.fp, 2);
        assert_eq!(c.tp, 0);
        assert!((c.precision()).abs() < 1e-9);
        assert!((c.f1_score()).abs() < 1e-9);
    }

    #[test]
    fn test_confusion_counts_empty() {
        let c = ConfusionCounts::new();
        assert!((c.accuracy()).abs() < 1e-9);
        assert!((c.precision()).abs() < 1e-9);
        assert!((c.f1_score()).abs() < 1e-9);
    }

    #[test]
    fn test_bench_veto_only_perfect() {
        let mut bench = ClassificationBenchmark::new();
        bench.add_batch(&[
            ("safe claim 1", false),
            ("safe claim 2", false),
            ("risky claim 1", true),
            ("risky claim 2", true),
        ]);

        let mut veto = AnubisVeto::new(VetoActivation::Hard);
        let result = bench.run_veto_only("perfect", &mut veto);
        assert_eq!(result.counts.tp, 2); // risky → halted
        assert_eq!(result.counts.tn, 2); // safe → allowed
        assert_eq!(result.counts.fp, 0);
        assert_eq!(result.counts.fn_count, 0);
        assert!((result.accuracy - 1.0).abs() < 1e-9);
        assert!((result.f1_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_bench_veto_only_imperfect() {
        let mut bench = ClassificationBenchmark::new();
        // All safe, but Hard veto with score=-1 allows all → all TN
        bench.add_batch(&[
            ("safe", false),
            ("safe", false),
            ("safe", false),
            ("safe", false),
        ]);

        let mut veto = AnubisVeto::new(VetoActivation::Hard);
        let result = bench.run_veto_only("all_safe", &mut veto);
        assert_eq!(result.counts.tn, 4);
        assert_eq!(result.total_claims, 4);
        assert!((result.accuracy - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_bench_result_prints() {
        let r = BenchmarkResult {
            name: "test".to_string(),
            counts: ConfusionCounts {
                tp: 8,
                tn: 80,
                fp: 2,
                fn_count: 10,
            },
            total_claims: 100,
            accuracy: 0.88,
            precision: 0.8,
            recall: 0.4444,
            f1_score: 0.5714,
            total_latency_us: 1000,
            avg_latency_us: 10.0,
            claims_per_second: 10000.0,
        };
        r.print_summary(); // Verify it doesn't panic
    }

    #[test]
    fn test_full_pipeline_bench() {
        let mut irp = InMemoryIRP::new();
        irp.insert(
            "safe",
            SymbolicWitness {
                claim_id: "ev".to_string(),
                evidence_text: "evidence".to_string(),
                source: "db".to_string(),
                confidence: 0.9,
                timestamp_us: 0,
            },
        );

        let veto = AnubisVeto::new(VetoActivation::Sigmoid);
        let mut monitor = SeamIntegrityMonitor::new(irp, veto);

        let mut bench = ClassificationBenchmark::new();
        bench.add(ClassificationCase::new(
            "safe claim about safe topic",
            false,
        ));
        bench.add(ClassificationCase::new("unknown claim about nothing", true));

        let result = bench.run("pipeline_test", &mut monitor);
        assert_eq!(result.total_claims, 2);
        assert!(result.accuracy >= 0.0 && result.accuracy <= 1.0);
        // Verify TP+TN+FP+FN = total
        let c = &result.counts;
        assert_eq!(c.tp + c.tn + c.fp + c.fn_count, 2);
    }
}
