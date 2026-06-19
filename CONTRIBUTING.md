# Contributing to DTPF

Thank you for your interest in the Desktop Task Presence Framework (DTPF). This guide covers local setup, development workflows, and pull request expectations.

## Prerequisites

- **Node.js 20+** and **pnpm 10+** (`corepack enable`)
- **Rust** stable (installed automatically by `pnpm install-deps` on Windows, or via [rustup.rs](https://rustup.rs))
- Platform-specific build tools:
  - **Windows:** Visual Studio Build Tools with the **Desktop development with C++** workload, WebView2 Runtime
  - **Linux:** WebKitGTK and appindicator dev packages (see [README.md](README.md))

## Setup

From the repository root:

```bash
pnpm install-deps
```

This runs the cross-platform installer (`scripts/install-deps.mjs`), which installs Node workspace dependencies and verifies or installs system tools (Rust, MSVC on Windows, etc.).

Alternatively, for a minimal Node-only install:

```bash
pnpm install
pnpm setup
```

## Development

Run the desktop agent and demo web app in separate terminals:

```bash
pnpm agent:dev      # Tauri desktop agent (Windows: scripts/agent-dev.ps1)
pnpm demo:dev       # examples/react-basic demo app (Vite)
```

On Linux/macOS you can start the agent with:

```bash
pnpm agent:dev:unix
```

On first connect, approve the demo app in the agent's native registration dialog.

Other useful commands:

```bash
pnpm build          # Build all packages and apps
pnpm lint           # Lint across the monorepo
pnpm test           # Run tests
```

Agent data is stored under `%LOCALAPPDATA%\dtpf\` on Windows and `~/.dtpf/` on Linux/macOS.

## Monorepo layout

| Path | Description |
|------|-------------|
| `apps/desktop-agent` | Tauri v2 desktop agent |
| `packages/sdk-core` | Framework-agnostic TypeScript SDK |
| `packages/sdk-react` | React hooks and context |
| `packages/shared-types` | Shared TypeScript types |
| `examples/react-basic` | Demo integration |

## Pull request guidelines

1. **Branch from `main`** and keep changes focused on a single concern.
2. **Describe what and why** in the PR description. Link related issues when applicable.
3. **Test locally** before submitting:
   - `pnpm lint` and `pnpm build` for SDK or demo changes
   - `pnpm agent:dev` plus `pnpm demo:dev` for end-to-end agent or integration work
   - `cargo test` in `apps/desktop-agent/src-tauri` for Rust changes
4. **Follow existing conventions** — match naming, formatting, and patterns in the files you touch.
5. **Do not commit secrets** — no API keys, tokens, or local agent data from `%LOCALAPPDATA%\dtpf\` or `~/.dtpf/`.
6. **CI must pass** — PRs run lint/build jobs and agent build checks on Ubuntu, Windows, and macOS (see `.github/workflows/ci.yml`).

## Releasing

See [RELEASE.md](RELEASE.md) for GitHub secrets setup, tagging, and verification steps.

## Reporting issues

- **Bugs:** use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- **Features:** use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- **Security:** see [SECURITY.md](SECURITY.md) — do not open public issues for vulnerabilities

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). By participating, you agree to uphold it.
