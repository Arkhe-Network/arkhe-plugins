// crates/arkhe-core/src/safety/symmetry_generator.rs
//! ARKHE-χ Fase 1 — Symmetry Generator
//!
//! Implementação baseada no artigo "Symmetry-Induced Weyl Nodes..."
//! O gerador de simetria avalia se o sistema está operando dentro
//! do Safety Manifold ℳ_safe.

/// Configuração do sistema (limites operacionais)
#[derive(Debug, Clone, PartialEq)]
pub struct SystemConfig {
    pub max_tokens: i64,
    pub max_agents: u32,
    pub min_fuel: i64,
    pub min_entropy: u32,
    pub max_rate_limit: i64,
    pub max_sandbox_fuel: i64, // Fix R1
    pub topological_gap_threshold: f64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max_tokens: 10_000,
            max_agents: 10,
            min_fuel: 100,
            min_entropy: 256,
            max_rate_limit: 100,
            max_sandbox_fuel: 1_000, // Fix R1
            topological_gap_threshold: 0.5,
        }
    }
}

/// Estado do sistema (SystemState)
#[derive(Debug, Clone, PartialEq)]
pub struct SystemState {
    pub config: SystemConfig,
    pub token_budget: i64,
    pub agent_count: u32,
    pub sandbox_fuel: i64,
    pub entropy_bits: u32,
    pub pii_scrubbed: bool,
    pub signature_valid: bool,
    pub rate_limit_remaining: i64,
    pub model_capability: u64, // Fix R6
    pub task_requirement: u64, // Fix R6
}

impl SystemState {
    pub fn safe(config: SystemConfig) -> Self {
        Self {
            config: config.clone(),
            token_budget: config.max_tokens / 2,
            agent_count: config.max_agents / 2,
            sandbox_fuel: config.max_sandbox_fuel / 2,
            entropy_bits: config.min_entropy * 2,
            pii_scrubbed: true,
            signature_valid: true,
            rate_limit_remaining: config.max_rate_limit / 2,
            model_capability: 0xFFFFFFFFFFFFFFFF, // Todos os bits
            task_requirement: 0x00000000000000FF,
        }
    }
}

/// Nível de criticidade do invariante
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvariantClass {
    Critical,   // Violável apenas em ℳ_safe (nunca fora)
    High,       // Degradê gracioso permitido
    Medium,     // Alerta + logging
    Low,        // Métrica apenas
}

/// Tipo de violação (Fix R4: Removido High)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationType {
    Critical { invariant_ids: Vec<String> },
}

impl ViolationType {
    pub fn invariant_id(&self) -> String {
        match self {
            ViolationType::Critical { invariant_ids } => invariant_ids.join(","),
        }
    }
}

/// Resultado de pertencimento ao manifold
#[derive(Debug, Clone, PartialEq)]
pub enum ManifoldResult {
    Inside,
    Degraded(Vec<(String, InvariantClass)>),
    Outside { violation: ViolationType, state: SystemState },
}

/// Segurança da transição (Fix R5: Adicionado Recovery)
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionSafety {
    Safe,                    // Dentro → Dentro
    CriticalEscape { violation: ViolationType, state: SystemState },  // Dentro → Fora (Weyl node)
    CascadeFailure { violation: ViolationType },  // Degradado → Fora (escape contínuo)
    Recovery,                // Fora → Dentro (circuit breaker half-open)
    Degraded { violations: Vec<String>, warning: String },
    Unsafe { reason: String }, // Fora → Fora, ou outros indefinidos
}

/// Trait para invariantes (Fix D1)
pub trait Invariant: Send + Sync {
    fn id(&self) -> &'static str;
    fn class(&self) -> InvariantClass;
    fn check(&self, state: &SystemState) -> bool;

    /// Computa a margem de segurança para este invariante (0.0 = fronteira, 1.0 = centro)
    fn margin(&self, state: &SystemState) -> f64 {
        if self.check(state) { 1.0 } else { 0.0 }
    }
}

/// Gerador de simetria 𝒫_safe
pub struct SymmetryGenerator {
    pub invariants: Vec<Box<dyn Invariant>>,
    pub config: SystemConfig,
}

impl SymmetryGenerator {
    pub fn new(invariants: Vec<Box<dyn Invariant>>, config: SystemConfig) -> Self {
        Self { invariants, config }
    }

    pub fn invariants(&self) -> &[Box<dyn Invariant>] {
        &self.invariants
    }

    /// Computa a "distância" até a fronteira do manifold (spectral gap).
    /// Fix C2: Usa fold(1.0, f64::min) em vez de média.
    pub fn compute_spectral_gap(&self, state: &SystemState) -> f64 {
        let margins: Vec<f64> = self.invariants.iter()
            .map(|inv| inv.margin(state))
            .collect();

        if margins.is_empty() {
            1.0
        } else {
            margins.into_iter().fold(1.0, f64::min)
        }
    }

    /// Verifica se um estado está em ℳ_safe
    pub fn is_in_manifold(&self, state: &SystemState) -> ManifoldResult {
        let mut degraded_violations = Vec::new();
        let mut critical_violations = Vec::new();

        for inv in &self.invariants {
            if !inv.check(state) {
                match inv.class() {
                    InvariantClass::Critical => {
                        critical_violations.push(inv.id().to_string());
                    }
                    _ => degraded_violations.push((inv.id().to_string(), inv.class())),
                }
            }
        }

        if !critical_violations.is_empty() {
            return ManifoldResult::Outside {
                violation: ViolationType::Critical { invariant_ids: critical_violations },
                state: state.clone(),
            };
        }

        if degraded_violations.is_empty() {
            ManifoldResult::Inside
        } else {
            ManifoldResult::Degraded(degraded_violations)
        }
    }

    /// Verifica se uma transição T preserva ℳ_safe (Fix C1)
    pub fn preserves_manifold(
        &self,
        from: &SystemState,
        to: &SystemState,
    ) -> TransitionSafety {
        match (self.is_in_manifold(from), self.is_in_manifold(to)) {
            (ManifoldResult::Inside, ManifoldResult::Inside) => {
                TransitionSafety::Safe
            }
            (ManifoldResult::Inside, ManifoldResult::Outside { violation, state }) => {
                TransitionSafety::CriticalEscape { violation, state }
            }
            (ManifoldResult::Inside, ManifoldResult::Degraded(violations)) => {
                TransitionSafety::Degraded {
                    violations: violations.into_iter().map(|(id, _)| id).collect(),
                    warning: "Transition resulted in degraded state".into(),
                }
            }
            (ManifoldResult::Degraded(_), ManifoldResult::Inside) => {
                TransitionSafety::Recovery // Recovery is fine
            }
            (ManifoldResult::Degraded(_), ManifoldResult::Outside { violation, state: _ }) => {
                TransitionSafety::CascadeFailure { violation }
            }
            (ManifoldResult::Degraded(_), ManifoldResult::Degraded(violations)) => {
                TransitionSafety::Degraded {
                    violations: violations.into_iter().map(|(id, _)| id).collect(),
                    warning: "State remains degraded".into(),
                }
            }
            (ManifoldResult::Outside { .. }, ManifoldResult::Inside) => {
                TransitionSafety::Recovery // Fix R5: Adicionado Recovery manual
            }
            (ManifoldResult::Outside { .. }, _) => {
                TransitionSafety::Unsafe { reason: "Initial state is outside the manifold (requires recovery)".into() }
            }
        }
    }
}
