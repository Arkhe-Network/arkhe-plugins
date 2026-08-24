use criterion::{black_box, criterion_group, criterion_main, Criterion};
use arkhe_core::safety::symmetry_generator::{SymmetryGenerator, SystemConfig, SystemState};
use arkhe_core::safety::invariants::all_invariants;

fn bench_is_in_manifold(c: &mut Criterion) {
    let gen = SymmetryGenerator::new(all_invariants(), SystemConfig::default());
    let state = SystemState::safe(SystemConfig::default());

    c.bench_function("is_in_manifold_safe", |b| {
        b.iter(|| gen.is_in_manifold(black_box(&state)))
    });
}

fn bench_compute_spectral_gap(c: &mut Criterion) {
    let gen = SymmetryGenerator::new(all_invariants(), SystemConfig::default());
    let state = SystemState::safe(SystemConfig::default());

    c.bench_function("compute_spectral_gap_safe", |b| {
        b.iter(|| gen.compute_spectral_gap(black_box(&state)))
    });
}

fn bench_preserves_manifold(c: &mut Criterion) {
    let gen = SymmetryGenerator::new(all_invariants(), SystemConfig::default());
    let from = SystemState::safe(SystemConfig::default());
    let to = SystemState::safe(SystemConfig::default());

    c.bench_function("preserves_manifold_safe", |b| {
        b.iter(|| gen.preserves_manifold(black_box(&from), black_box(&to)))
    });
}

criterion_group!(benches, bench_is_in_manifold, bench_compute_spectral_gap, bench_preserves_manifold);
criterion_main!(benches);
