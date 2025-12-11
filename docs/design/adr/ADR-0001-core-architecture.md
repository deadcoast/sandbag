# ADR-0001: Core Architecture and Tech Stack

> Navigation: [Documentation Hub](../../00_MOC.md) | [Project Root](../../../00_MOC.md) | [Cursor Rules](../../../.cursorrules)

## Status
Accepted

## Context
Sandbag requires a performant, safe CLI that integrates mathematical analysis and configuration management across linters.

## Decision
- Language: Rust
- CLI: `clap`
- Async FS: `tokio`
- Parsing/Regex: `regex`
- Serialization: `serde`
- Performance: `rayon`, benchmarks with `criterion`
- Data structures: `indexmap`, `dashmap`
- Error handling: `anyhow`/`thiserror`
- Config AST: custom types under `src/config/`
- UI: enhanced CLI with color/progress in `src/ui/`

## Consequences
- Strong safety and performance guarantees
- Clear module boundaries and SRP across `core`, `config`, `linters`, `ui`, `performance`
- Straightforward extensibility via linter handlers and plugin architecture

## Alternatives Considered
- Python CLI: faster iteration but weaker performance/safety
- Node.js CLI: ecosystem benefits but less suitable for compute-heavy tasks

## References
- `docs/architecture.md`
- `docs/implementation.md`
- `docs/mathematics.md`
