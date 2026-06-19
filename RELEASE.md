# Releasing DTPF

This guide covers the one-time setup and per-release steps for shipping npm packages and desktop agent binaries.

## Canonical repository

All releases, docs, and updater endpoints use:

**https://github.com/Parthmh361/Desktop-Task-Presence-Framework**

## One-time GitHub Actions secrets

In **GitHub → Settings → Secrets and variables → Actions** on the repository above, add:

| Secret | Value |
|--------|--------|
| `NPM_TOKEN` | npm automation token with publish access to the `@dtpf` scope ([create token](https://www.npmjs.com/settings/~tokens)) |
| `TAURI_PRIVATE_KEY` | Full contents of `~/.tauri/dtpf-agent.key` (from `pnpm tauri signer generate`) |
| `TAURI_KEY_PASSWORD` | Password chosen when generating the signing key |

The public key must match `plugins.updater.pubkey` in `apps/desktop-agent/src-tauri/tauri.conf.json`. Never commit the private key.

### Generate Tauri signing keys

```powershell
cd apps/desktop-agent
pnpm tauri signer generate -w "$env:USERPROFILE\.tauri\dtpf-agent.key"
```

Paste the printed public key into `tauri.conf.json` if it differs from the current value.

## GitHub Pages (docs)

The docs site must be deployed once before release links work. Do this **once** on the repository:

1. Open **Settings → Pages → Build and deployment**
2. Set **Source** to **GitHub Actions** (not “Deploy from a branch”)
3. Go to **Actions → Deploy Docs → Run workflow** (or push any commit to `main`)

The workflow [`.github/workflows/docs.yml`](.github/workflows/docs.yml) builds Docusaurus from `apps/docs/` and publishes on every push to `main`.

Live URL: https://parthmh361.github.io/Desktop-Task-Presence-Framework/

If you see a GitHub Pages 404, Pages is not enabled or the Deploy Docs workflow has not completed successfully yet. Use the [docs source](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/tree/main/apps/docs/docs) as a fallback.

## Pre-release checklist

```bash
pnpm release:check   # lint + test + build
```

Confirm CI is green on `main` before tagging.

## Cut a release

1. Add a changeset if npm package versions need bumping:

   ```bash
   pnpm changeset
   pnpm changeset version
   ```

2. Bump the agent version in `apps/desktop-agent/src-tauri/tauri.conf.json` and `Cargo.toml` to match the tag.

3. Commit and push to `main`:

   ```bash
   git add .
   git commit -m "chore: prepare v1.0.x release"
   git push origin main
   ```

4. Tag and push (triggers `.github/workflows/release.yml`):

   ```bash
   git tag v1.0.x
   git push origin v1.0.x
   ```

## Verify after release

- [GitHub Releases](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/releases) has Win/Linux/macOS installers and `latest.json`
- npm packages updated at https://www.npmjs.com/org/dtpf
- Tray **Check for Updates** reports "latest version" (not a fetch error)
- Smoke test: install agent + `npm i @dtpf/sdk-react` + create a sticky from a demo app

## Manual npm publish (fallback)

If the release workflow npm job fails:

```bash
pnpm build --filter @dtpf/shared-types --filter @dtpf/sdk-core --filter @dtpf/sdk-react --filter @dtpf/sdk-vanilla
pnpm changeset publish
```
