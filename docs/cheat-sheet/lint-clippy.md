# CLIPPY CHEATSHEET

> Lint the entire directory

```bash
cargo clippy
```

---

> Raw output to clippy_output.txt file

```bash
cargo clippy --all-targets --all-features -- -D warnings >clippy_output.txt 2>&1
```

---

> Output Lint to clippy_output.json file

```bash
cargo clippy --message-format=json -- -D warnings >clippy_output.json
```

---
