use arkhe_core::safety::symmetry_generator::{
    SymmetryGenerator, SystemConfig, SystemState, TransitionSafety, ManifoldResult,
    ViolationType,
};
use arkhe_core::safety::invariants::all_invariants;

fn safe_state() -> SystemState {
    SystemState::safe(SystemConfig::default())
}

fn generator() -> SymmetryGenerator {
    SymmetryGenerator::new(all_invariants(), SystemConfig::default())
}

#[test]
fn test_safe_state_is_inside() {
    let gen = generator();
    let state = safe_state();
    assert_eq!(gen.is_in_manifold(&state), ManifoldResult::Inside);
}

#[test]
fn test_critical_violation_is_outside() {
    let gen = generator();
    let mut state = safe_state();
    state.token_budget = -1; // I-01 is Critical

    let result = gen.is_in_manifold(&state);
    assert!(matches!(result, ManifoldResult::Outside { violation: ViolationType::Critical { .. }, .. }));
}

#[test]
fn test_high_violation_is_degraded() {
    let gen = generator();
    let mut state = safe_state();
    state.agent_count = state.config.max_agents + 1; // I-02 is High

    let result = gen.is_in_manifold(&state);
    assert!(matches!(result, ManifoldResult::Degraded(_)));
}

#[test]
fn test_preserves_manifold_safe() {
    let gen = generator();
    let from = safe_state();
    let mut to = safe_state();
    to.token_budget = 4000; // Still valid

    assert_eq!(gen.preserves_manifold(&from, &to), TransitionSafety::Safe);
}

#[test]
fn test_critical_escape_transition() {
    let gen = generator();
    let from = safe_state();
    let mut to = safe_state();
    to.token_budget = -1; // Escapes manifold

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::CriticalEscape { .. }));
}

#[test]
fn test_degraded_to_degraded_transition() {
    let gen = generator();
    let mut from = safe_state();
    from.agent_count = from.config.max_agents + 1; // Degraded (I-02 HIGH violado)
    let mut to = safe_state();
    to.rate_limit_remaining = -1; // Fix T1: I-07 violado (HIGH) -> Degraded

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::Degraded { .. }));
}

#[test]
fn test_cascade_failure_transition() {
    let gen = generator();
    let mut from = safe_state();
    from.agent_count = from.config.max_agents + 1; // Degraded (I-02 HIGH violado)
    let mut to = safe_state();
    to.token_budget = -1; // Fix T2: I-01 CRITICAL violado -> Outside

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::CascadeFailure { .. }));
}

#[test]
fn test_recovery_transition() {
    let gen = generator();
    let mut from = safe_state();
    from.token_budget = -1; // Outside
    let to = safe_state(); // Inside

    let result = gen.preserves_manifold(&from, &to);
    assert_eq!(result, TransitionSafety::Recovery);
}

#[test]
fn test_unsafe_transition() {
    let gen = generator();
    let mut from = safe_state();
    from.token_budget = -1; // Outside
    let mut to = safe_state();
    to.token_budget = -1; // Outside

    let result = gen.preserves_manifold(&from, &to);
    assert!(matches!(result, TransitionSafety::Unsafe { .. }));
}

#[test]
fn test_compute_spectral_gap() {
    let gen = generator();
    let state = safe_state();
    let gap = gen.compute_spectral_gap(&state);
    assert!(gap > 0.0);
    assert!(gap <= 1.0);
}
