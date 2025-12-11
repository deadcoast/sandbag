# Examples: Best Practices & Troubleshooting

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## Best Practices

- Configuration management: backup, version control, test in dev
- Performance: cache, parallelize, monitor memory, profile regularly
- Error handling: retries, clear messages, logs, graceful handling
- Testing: unit, integration, performance, coverage > 80%

## Troubleshooting

- Rule extraction fails: check formats, regex, valid IDs
- Performance issues: enable caching/parallel, profile hotspots
- Config conflicts: use conflict resolution, validate before apply, backup

## Debug Mode

```bash
RUST_LOG=debug sandbag scan /path/to/project
```
