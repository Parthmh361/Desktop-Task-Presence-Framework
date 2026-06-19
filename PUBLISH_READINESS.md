# DTPF Publish Readiness Report

**Date:** 2026-06-19  
**Repository:** https://github.com/Parthmh361/Desktop-Task-Presence-Framework

## Executive summary

DTPF is **published and usable** as an open-source local-first framework:

- Four npm packages are live under the `@dtpf` scope (v1.0.0).
- Signed desktop agent binaries are available on GitHub Releases (v1.0.1).
- Documentation is deployed to GitHub Pages.

This report documents what is shipped, what was cleaned for a public repo, and optional follow-ups.

---

## Consumer checklist

External developers can complete this flow today:

| Step | Action | Status |
|------|--------|--------|
| 1 | Install SDK: `npm install @dtpf/sdk-react` (or `@dtpf/sdk-vanilla`) | npm live |
| 2 | Download agent for your OS from [Releases](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases) | v1.0.1 |
| 3 | Follow [Quick start](https://parthmh361.github.io/Desktop-Task-Presence-Framework/quickstart) | Docs live |
| 4 | Run a demo (`examples/react-basic`, `vanilla-js`, or `nextjs-tasks`) against the agent | In repo |
| 5 | Report issues via GitHub templates | Configured |

---

## Maintainer checklist

| Item | Location / command |
|------|-------------------|
| Pre-release verification | `pnpm release:check` (lint + test + build) |
| GitHub secrets | `NPM_TOKEN`, `TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD` — see [RELEASE.md](RELEASE.md) |
| Tag a release | `git tag vX.Y.Z && git push origin vX.Y.Z` |
| npm publish | Automated in `.github/workflows/release.yml` on tag push |
| Agent binaries | Built by `tauri-action` on tag push |
| Docs deploy | Automatic on push to `main` via `.github/workflows/docs.yml` |
| GitHub Pages source | Settings → Pages → **GitHub Actions** |

---

## Shipped artifacts

| Artifact | Version | URL |
|----------|---------|-----|
| `@dtpf/shared-types` | 1.0.0 | https://www.npmjs.com/package/@dtpf/shared-types |
| `@dtpf/sdk-core` | 1.0.0 | https://www.npmjs.com/package/@dtpf/sdk-core |
| `@dtpf/sdk-react` | 1.0.0 | https://www.npmjs.com/package/@dtpf/sdk-react |
| `@dtpf/sdk-vanilla` | 1.0.0 | https://www.npmjs.com/package/@dtpf/sdk-vanilla |
| DTPF Agent (Win/Linux/macOS) | 1.0.1 | https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases |
| Documentation site | latest `main` | https://parthmh361.github.io/Desktop-Task-Presence-Framework/ |

---

## Version matrix

| Component | Published / released | Repo source (main) |
|-----------|----------------------|---------------------|
| npm packages | **1.0.0** | **1.0.1** in `packages/*/package.json` |
| Desktop agent | **1.0.1** (GitHub tag) | **1.0.1** in `tauri.conf.json` / `Cargo.toml` |

SDK 1.0.0 is compatible with agent 1.0.1. To align npm with repo versions, publish 1.0.1 on the next changeset release (optional).

---

## Repo cleanliness (this hygiene pass)

| Item | Before | After |
|------|--------|-------|
| `.cursor/` (IDE plans, duplicates) | Tracked in git | Removed + added to `.gitignore` |
| `apps/docs/.docusaurus/` (build cache) | 22 tracked files | Untracked (already in `.gitignore`) |
| `examples/nextjs-tasks/.next/` (Next build) | 77 tracked files | Untracked (already in `.gitignore`) |
| Architecture spec filename | `desktop-task-presence-framework.md` | Renamed to [ARCHITECTURE.md](ARCHITECTURE.md) |
| Cursor `Co-authored-by` trailers | In commit history | Removed via `git filter-repo` |

**Note:** `.cursor/` on disk is ignored locally. Only commit `.cursor/rules` if you intentionally want shared AI conventions — never commit `.cursor/plans/` or draft copies.

---

## OSS hygiene files

| File | Purpose |
|------|---------|
| [LICENSE](LICENSE) | MIT |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contributor guide |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community standards |
| [SECURITY.md](SECURITY.md) | Localhost threat model |
| [RELEASE.md](RELEASE.md) | Release maintainer guide |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Full technical spec |
| `.github/ISSUE_TEMPLATE/` | Bug + feature templates |
| `.github/PULL_REQUEST_TEMPLATE.md` | PR template |
| `.github/workflows/ci.yml` | Lint, test, multi-OS agent build |
| `.github/workflows/release.yml` | npm + signed agent release |
| `.github/workflows/docs.yml` | GitHub Pages deploy |

---

## CI coverage

| Job | Runs on |
|-----|---------|
| `lint-and-build` | ubuntu-latest |
| `test` | ubuntu-latest (includes `cargo test` for agent) |
| `build-agent` | ubuntu, windows, macos |
| `Deploy Docs` | push to `main` |
| `Release` | tag `v*.*.*` |

---

## Known gaps (documented, not blocking publish)

These were explicitly deferred per the local-first OSS scope:

- **Cloud sync** — `apps/desktop-agent/src-tauri/src/sync/mod.rs` remains a stub
- **SQLCipher** — plain SQLite; TODO in `db/mod.rs`
- **Preferences UI** — tray menu shows “coming soon”
- **macOS sticky transparency** — opaque windows (no `macos-private-api`)
- **Playwright e2e** — health-check skeleton only; not wired into CI
- **Sentry** — optional observability; not integrated

---

## Optional follow-ups

1. **Publish npm 1.0.1** — run `pnpm changeset version` + tag if you want npm aligned with repo package.json
2. **npm README refresh** — republish packages so npm pages show package READMEs (some show “No README” on 1.0.0)
3. **Expand e2e** — full sticky create flow with agent as CI service
4. **API reference page** — add generated SDK API section to `apps/docs`

---

## Quick verification commands

```bash
pnpm release:check          # lint + test + build
pnpm --filter @dtpf/docs build
pnpm agent:dev              # terminal 1
pnpm demo:dev               # terminal 2
```

Agent health (when running): `curl http://127.0.0.1:7842/health`
