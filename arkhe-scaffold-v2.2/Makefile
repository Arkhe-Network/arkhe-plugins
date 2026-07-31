.PHONY: check test test-verbose clippy fmt clean lean-check

# Default: check all Rust crates compile
check:
	cargo check --workspace --all-targets

# Run all Rust tests
test:
	cargo test --workspace

# Run tests with output
test-verbose:
	cargo test --workspace -- --nocapture

# Clippy lints
clippy:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Clean build artifacts
clean:
	cargo clean --workspace

# Lean check (requires elan/lake)
lean-check:
	cd arkhe-spec && lake build

# Full CI pipeline
ci: fmt-check clippy test
	@echo "✅ All checks passed"
