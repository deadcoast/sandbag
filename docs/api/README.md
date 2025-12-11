# API Reference

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

This folder hosts generated or curated API documentation.

## Index

- Core types overview: see API section in `../README.md`
- Modules:
  - Core: `src/core/` (rule extraction, similarity, confidence)
  - Config: `src/config/` (AST, parser, writer, backups)
  - Linters: `src/linters/` (handlers, registry, plugins)
  - Performance: `src/performance/`
  - UI: `src/ui/`
  - Utils: `src/utils/`

## Generating API Docs

- Rustdoc: `cargo doc --no-deps --document-private-items`
- Output can be published under this folder or served via GitHub Pages.

Status: Curated index. Expand with generated docs as needed.
