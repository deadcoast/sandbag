# Examples: Integration

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## CI/CD Pipeline Integration (GitHub Actions)

```yaml
# .github/workflows/sandbag.yml
name: Sandbag Linter Configuration
on:
  pull_request:
    branches: [main]
jobs:
  sandbag:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install Sandbag
        run: cargo install --path .
      - name: Run Sandbag Analysis
        run: sandbag scan . --output-format json > linter-analysis.json
      - name: Generate Configuration
        run: sandbag config generate --input linter-analysis.json --format json > .sandbag-config.json
      - name: Commit Configuration
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add .sandbag-config.json
          git commit -m "Update linter configuration" || exit 0
```

## IDE Integration (VS Code)

```json
{
  "sandbag.enabled": true,
  "sandbag.configFile": ".sandbag-config.json",
  "sandbag.autoApply": true,
  "sandbag.linters": ["markdownlint", "eslint", "prettier"],
  "sandbag.performanceMode": true
}
```
