# Quality Gates (Estratégia de Verificação ASI‑Grade v2.0)

Este guia documenta os comandos e procedimentos para a Estratégia de Cargo Check ASI-Grade.

## Instalação de Ferramentas

Para instalar todas as ferramentas necessárias para os quality gates, execute:

```bash
cargo install cargo-llvm-cov cargo-insta cargo-deny cargo-audit cargo-semver-checks
```

## Executar Pre-Commit Localmente

Antes de criar um Pull Request, execute o pipeline de nível 1 (pre-commit) localmente:

```bash
cargo xtask pre-commit
```

## Revisão de Snapshots

Se os testes de snapshot falharem devido a mudanças intencionais na saída, você pode revisá-los e atualizá-los executando:

```bash
cargo insta review
```

## Cobertura de Código

Para gerar relatórios de cobertura, utilizamos o `cargo llvm-cov`. Após executar testes (ex: via `cargo xtask ci` ou `cargo xtask pre-commit`), os relatórios LCOV podem ser analisados para identificar áreas com baixa cobertura de código. A meta é ter uma cobertura de testes > 80%.
