# Arkhe Cathedral — Scaffold v2.2

Agent evaluation framework with hypergraph-based claim verification.

## Status

- **Rust**: ✅ Compilable, 70+ testes, 0 warnings (clippy)
- **Lean 4**: ⚠️ Arquivos .lean existem mas não são compiláveis sem Mathlib full
- **Processing**: ✅ Sketch completo para visualização

## Quick Start

```bash
cargo test --workspace
```

## Architecture

```
arkhe-core          Shared primitives (activation, loss, symbolic IRP)
arkhe-hypergraph    Hypergraph + ML extensions (veto, agent, QPL, bench)
arkhe-spec          Lean 4 formalization (independent of Rust crates)
sketch_arkhe_agent  Processing 4 real-time visualizer
```

## Honesty Policy

This codebase does NOT implement:
- Quantum physics (QPL is a structural analogy)
- Algebraic topology (no homology computed)
- Geometric analysis (no Ricci flow)

Names are chosen for information-flow clarity, not mathematical equivalence.

## Score

Audited v20.2: 92/100
See commit history for audit trail.
