---
phase: 5
slug: flatpak-distirbution-and-automatic-update
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-22
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in `#[test]`) |
| **Config file** | Cargo.toml (default test profile) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test --all` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test --all`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | DIST-01, DIST-04 | smoke | `test -f flatpak/io.github.maartenp.puuzel.yml && grep -q "buildsystem: simple" flatpak/io.github.maartenp.puuzel.yml && grep -q "CARGO_NET_OFFLINE" flatpak/io.github.maartenp.puuzel.yml && echo PASS \|\| echo FAIL` | N/A (file checks) | pending |
| 5-01-02 | 01 | 1 | DIST-04 | smoke | `test -x release.sh && grep -q 'git tag' release.sh && ! grep -q 'source = "git+' Cargo.lock && echo PASS \|\| echo FAIL` | N/A (file checks) | pending |
| 5-02-01 | 02 | 1 | DIST-02 | compile | `cargo check 2>&1 \| tail -5` | N/A (compile check) | pending |
| 5-02-02 | 02 | 1 | DIST-02 | compile | `cargo check 2>&1 \| tail -5` | N/A (compile check) | pending |
| 5-03-01 | 03 | 2 | DIST-01, DIST-02 | smoke | `test -f .github/workflows/release.yml && grep -q "upload-pages-artifact.*false" .github/workflows/release.yml && grep -q "version.txt" .github/workflows/release.yml && grep -q "upload-pages-artifact@v3" .github/workflows/release.yml && echo PASS \|\| echo FAIL` | N/A (file checks) | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No test scaffolds needed:
- Plan 01 tasks produce config files verified by file-existence and grep checks
- Plan 02 tasks produce Rust code verified by `cargo check` (compilation)
- Plan 03 tasks produce CI workflow files verified by file-existence and grep checks
- DIST-04 (no git deps) verified by grepping Cargo.lock

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Flatpak builds successfully in CI | DIST-01 | Requires GitHub Actions runner with Flatpak SDK | Push a v* tag, watch Actions tab, confirm green |
| version.txt reachable on GitHub Pages | DIST-02 | Requires live GitHub Pages deployment | After CI deploy: `curl https://maartenp.github.io/puuzel/version.txt` returns version |
| .flatpakref install works on Linux Mint | DIST-02 | Requires dad's machine with GNOME Software | Double-click .flatpakref, confirm install dialog appears |
| Update notification appears in app | DIST-02 | Requires running Flatpak-installed app with outdated version | Install old version, push new tag, run `flatpak update`, verify notification |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-22
