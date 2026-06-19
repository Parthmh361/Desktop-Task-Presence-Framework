# Security Policy

## Supported versions

Security fixes are applied to the latest release on the `main` branch. Older releases are not actively supported.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Report security issues privately by opening a [GitHub Security Advisory](https://github.com/Parthmh361/Desktop-Task-Presence-Framework/security/advisories/new) or contacting the maintainers through the contact information in the repository. Include:

- A description of the vulnerability and its impact
- Steps to reproduce
- Affected components (agent, SDK, or both)
- Your platform (Windows, Linux, macOS)

We aim to acknowledge reports within 72 hours and will coordinate disclosure once a fix is available.

## Threat model

DTPF is designed as a **localhost-only bridge** between web applications and a native desktop agent. The agent exposes a REST and WebSocket API bound exclusively to loopback (`127.0.0.1`, ports 7842–7844). It is **not** intended to accept remote network traffic.

### In scope

| Threat | Mitigation |
|--------|------------|
| A malicious website creates sticky notes without user consent | Token-based authentication with explicit user approval on first registration (`POST /auth/register`) |
| Another local process calls the agent API | HMAC-signed bearer tokens; requests must originate from `127.0.0.1` or `::1`; `X-DTPF-App-ID` header required |
| A registered web app acts from the wrong origin | `Origin` header validated against the origin recorded at registration |
| A rogue web app floods the agent | Per-`app_id` sliding-window rate limit (100 requests/minute default) |
| Auth token stolen from browser storage | Tokens are scoped to `(appId + origin)`; a token registered for one origin cannot be reused from another |
| HMAC signing key exposure | 32-byte secret stored in the OS keychain (Keychain on macOS, Credential Manager on Windows, libsecret on Linux) with file fallback under the agent data directory |

### Out of scope

- **Remote network attacks** — the agent does not bind to `0.0.0.0` and is not reachable from other machines by design.
- **Malware with local user privileges** — a process running as the same user can read local files, memory, and browser storage. DTPF mitigates casual abuse but cannot defend against fully compromised local environments.
- **Physical access** — unattended machines with an unlocked session are outside the threat model.

## Localhost API security

The agent enforces the following on authenticated routes:

1. **`Authorization: Bearer <token>`** — token must match a registered app in the agent database
2. **`X-DTPF-App-ID`** — must match the app ID associated with the token
3. **`Origin`** — when present, must match the origin registered for that app
4. **Client address** — must be localhost (`127.0.0.1` or `::1`)

Registration requires user approval through a native dialog before a token is issued.

### Token design

```
Token = HMAC-SHA256(appId + ":" + origin + ":" + created_at, secret_key)
```

The `secret_key` is generated once and stored in the OS keychain. Tokens are long-lived by default and can be revoked by the user from the agent tray menu.

### Network binding

The agent binds to `127.0.0.1` only, never `0.0.0.0`. A lock file under the agent data directory records the active port for SDK discovery.

## Known limitations

- **Database encryption at rest** is planned (SQLCipher) but not yet enabled. Task data is stored in a local SQLite database under the agent data directory. Protect filesystem access accordingly.
- **CORS** is permissive at the HTTP layer because authentication and origin checks are enforced by the auth middleware, not browser CORS alone. Do not expose the agent beyond localhost.
- **macOS** agent support is in early stages; security properties above apply once the full agent is available on that platform.

## Recommendations for integrators

- Register a unique, stable `appId` per application.
- Store tokens in `localStorage` or another browser storage mechanism only for your registered origin.
- Never proxy agent requests through a remote server — keep all SDK-to-agent traffic on the user's machine.
- Revoke tokens for apps you no longer trust via the agent tray menu.

## Secure development

When contributing security-sensitive changes:

- Run `cargo test` in `apps/desktop-agent/src-tauri` for auth and server changes.
- Test registration, token validation, origin mismatch, and rate-limit behavior with `pnpm agent:dev` and `pnpm demo:dev`.
- Do not log tokens, HMAC secrets, or task content in debug output.
