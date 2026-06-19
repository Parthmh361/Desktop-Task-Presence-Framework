# Desktop Task Presence Framework (DTPF)
### Complete Architecture & Technical Plan — Production-Grade Open Source

---

## SECTION 1: PRODUCT VISION

### 1.1 Definition

The **Desktop Task Presence Framework (DTPF)** is an open-source, cross-platform framework that bridges web applications and the native desktop environment. It enables any web app — built in React, Next.js, Angular, Vue, or vanilla JS — to create, manage, and synchronize native desktop sticky notes, task overlays, and always-on-top reminders through a locally installed, lightweight desktop agent.

The framework consists of three coordinated layers:

- A **frontend SDK** that web apps use to issue commands
- A **desktop agent** (system tray app + background daemon) that executes those commands
- A **local communication layer** connecting them securely over localhost

### 1.2 Target Users

| Persona | Description |
|---|---|
| **Enterprise SaaS Teams** | Companies with internal task management tools (e.g., custom Jira dashboards, field operations tools) who want desktop-level task presence |
| **Indie SaaS Developers** | Builders of productivity tools who want desktop stickies without building a native app |
| **Remote-first Teams** | Distributed teams who need persistent, cross-session reminders beyond browser tabs |
| **Developers (Primary OSS Adopters)** | Engineers who will integrate DTPF into their web apps and contribute back |
| **Power Users** | Individuals who self-host or use DTPF as a personal productivity layer |

### 1.3 Primary Use Cases

1. **Task acceptance → sticky note**: User accepts a Jira-style task; a sticky note appears on the desktop immediately
2. **Meeting reminders**: A calendar web app creates an always-on-top reminder 10 minutes before a meeting
3. **SLA countdown timers**: A support dashboard creates a real-time countdown sticky for critical tickets
4. **Field operations checklists**: An ops dashboard pins a checklist to the desktop for a field worker
5. **Onboarding steps**: A product onboarding wizard places step-by-step guides on the desktop
6. **Personal focus sessions**: User starts a Pomodoro timer in a web app; a persistent overlay tracks time

### 1.4 Comparison with Existing Tools

| Tool | What it does | Why it falls short for DTPF's purpose |
|---|---|---|
| **Microsoft Sticky Notes** | Native app for manual stickies | No programmatic API; not web-app accessible; no external sync |
| **Jira / Asana / Trello** | Web-based task management | Lives in the browser; disappears when tab closes; no desktop presence |
| **Notion** | Workspace + docs + tasks | Browser-only; no desktop overlay capability |
| **Slack Reminders** | Notification-based reminders | Ephemeral push notifications; not persistent overlays; closes and is forgotten |
| **Raycast / Alfred** | Launcher + scripting for power users | Requires manual invocation; not programmable from web apps |
| **Taskbar badges / OS Notifications** | OS-level transient alerts | Dismissed instantly; no persistence; no content management lifecycle |

### 1.5 Unique Value Proposition

> **DTPF is the only framework that gives web applications first-class native desktop presence — programmatically, persistently, and offline-safely — without requiring the web app to become a desktop app.**

Key differentiators:

- **SDK-first**: A web developer with zero Electron knowledge can add `createStickyTask()` to their app in under an hour
- **Framework agnostic**: Works with any frontend stack
- **Lifecycle-aware**: Stickies sync with task state (created → updated → completed → removed)
- **Survives restarts**: Stickies persist across browser close, OS reboot
- **Offline-first**: Works without a cloud backend
- **Open source + self-hostable**: No vendor lock-in

---

## SECTION 2: SYSTEM ARCHITECTURE

### 2.1 High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        WEB APPLICATION                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              DTPF Frontend SDK                        │   │
│  │  createStickyTask() / updateStickyTask() / ...        │   │
│  │  Local Discovery → Localhost HTTP + WebSocket          │   │
│  └──────────────────────────┬───────────────────────────┘   │
└─────────────────────────────┼────────────────────────────────┘
                              │ HTTP (REST) + WebSocket
                              │ localhost:7842
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                     DESKTOP AGENT (Tauri)                      │
│                                                               │
│  ┌─────────────────┐   ┌──────────────────┐                 │
│  │  System Tray     │   │  HTTP/WS Server   │                │
│  │  (show/hide)     │   │  (NestJS/Axum)    │                │
│  └─────────────────┘   └────────┬─────────┘                 │
│                                 │                             │
│  ┌─────────────────────────────▼──────────────────────────┐ │
│  │              Task Manager (Core Daemon Logic)            │ │
│  │  CRUD tasks · resolve conflicts · manage windows         │ │
│  └───────────────┬──────────────────────────┬─────────────┘ │
│                  │                           │               │
│  ┌───────────────▼──────┐   ┌───────────────▼────────────┐  │
│  │  SQLite Persistence   │   │  Native Window Manager      │  │
│  │  (encrypted at rest)  │   │  (sticky notes / overlays)  │  │
│  └──────────────────────┘   └────────────────────────────┘  │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Startup Registration · Multi-monitor · Auto-update    │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                              │ Optional
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              OPTIONAL CLOUD SYNC BACKEND                      │
│          (NestJS API + PostgreSQL + Redis)                     │
│    Used only for cross-device sync / team features            │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Frontend SDK Architecture

The SDK is a thin wrapper with three responsibilities:

1. **Discovery**: Find the local desktop agent's port
2. **Command issuing**: Send task lifecycle commands over REST/WebSocket
3. **Event subscription**: Receive updates from the agent (task dismissed, reminder triggered)

```
packages/sdk-core/
  ├── discovery.ts        (agent discovery via localhost probe)
  ├── http-client.ts      (REST commands)
  ├── ws-client.ts        (real-time event subscription)
  ├── auth.ts             (token handshake)
  └── types.ts            (shared TypeScript interfaces)

packages/sdk-react/
  ├── hooks/
  │   ├── useStickyTask.ts
  │   ├── useTaskEvents.ts
  │   └── useAgentStatus.ts
  └── context/
      └── DTPFProvider.tsx

packages/sdk-vanilla/
  └── index.ts            (no-framework SDK, ESM + CJS)
```

### 2.3 Desktop Agent: Electron vs. Tauri

| Criterion | Electron | Tauri |
|---|---|---|
| **Bundle size** | 80–150 MB | 5–15 MB |
| **Memory footprint** | High (ships Chromium) | Very low (uses OS WebView) |
| **Performance** | Moderate | Excellent (Rust backend) |
| **Native API access** | Via Node.js + native modules | Via Rust + Tauri plugins |
| **Distribution** | Well-understood; electron-builder | cargo-based; maturing ecosystem |
| **WebView consistency** | Consistent (Chromium everywhere) | OS WebView (minor rendering differences) |
| **Startup time** | ~2–3s | <500ms |
| **Cross-platform** | Excellent | Excellent |
| **Community / ecosystem** | Very mature | Growing rapidly, very active |
| **Security** | Process isolation via sandbox | Rust memory safety + capability system |

**Recommendation: Tauri v2**

For a background daemon / system tray app, Tauri is the correct choice. The agent does not need to render a complex web UI — its native windows (sticky notes) are simple, lightweight overlays. Tauri's Rust backend gives us low memory footprint, fast startup, and a strong security model. The 5–15 MB installer is a meaningful adoption advantage over Electron's 80+ MB.

The only caveat: the sticky note windows in Tauri use the OS WebView (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). Since sticky content is simple HTML/CSS, rendering differences are manageable.

### 2.4 Backend Communication

```
Web App SDK ──(REST)──▶ Desktop Agent
           ──(WS)───▶  Desktop Agent (persistent bidirectional channel)

Agent internal: Rust command pattern via Tauri IPC
Optional cloud:  REST/WebSocket to NestJS cloud API
```

**Local Communication Design**

- **REST over localhost (HTTP/7842)**: For task CRUD operations (idempotent, easy to debug)
- **WebSocket (ws://localhost:7842/events)**: For real-time event streaming (task dismissed, reminder fired)
- **Port discovery**: The agent writes its active port to a well-known file (`~/.dtpf/agent.lock`) so the SDK can find it even if the port changes
- **No gRPC for local**: gRPC adds complexity without benefit on localhost; plain HTTP is sufficient

**Cloud Communication (Optional)**

- WebSocket for real-time sync across devices
- REST for CRUD when WebSocket is unavailable

### 2.5 Persistence Layer

| Option | Pros | Cons |
|---|---|---|
| **SQLite** | ACID, fast, file-based, widely supported | Requires native binding |
| **IndexedDB** | Browser-native | Browser-only; not accessible to Tauri Rust backend |
| **Local JSON** | Simple | No transactions; corruption risk on crash |
| **PostgreSQL** | Full relational | Overkill; requires server process |

**Recommendation: SQLite via `sqlx` (Rust)**

SQLite is the ideal choice for the desktop agent. It is ACID-compliant, file-based, and requires no separate server process. Using `sqlx` in the Rust backend gives us compile-time query validation. The database file lives at `~/.dtpf/tasks.db` and is encrypted at rest using SQLCipher.

Schema:

```sql
CREATE TABLE tasks (
  id            TEXT PRIMARY KEY,    -- UUID
  source_app_id TEXT NOT NULL,        -- which web app created it
  title         TEXT NOT NULL,
  body          TEXT,
  status        TEXT DEFAULT 'active', -- active | completed | dismissed | snoozed
  priority      INTEGER DEFAULT 0,
  color         TEXT DEFAULT '#FFE066',
  position_x    INTEGER,
  position_y    INTEGER,
  monitor_id    TEXT,
  remind_at     INTEGER,             -- Unix timestamp
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  synced_at     INTEGER,             -- last cloud sync timestamp
  metadata      TEXT                 -- JSON blob for app-specific data
);

CREATE TABLE app_registrations (
  app_id        TEXT PRIMARY KEY,
  app_name      TEXT NOT NULL,
  origin        TEXT NOT NULL,       -- allowed localhost origin
  token         TEXT NOT NULL,       -- HMAC token
  created_at    INTEGER NOT NULL,
  last_seen_at  INTEGER NOT NULL
);
```

### 2.6 Synchronization Design

```
┌─────────────────────────────────────────────────────────┐
│  SYNC STATE MACHINE (per task)                           │
│                                                          │
│  [local_only] ──push──▶ [synced]                        │
│  [synced]     ◀──pull── [synced]                        │
│  [conflict]   ──resolve─▶ [synced]                      │
│  [offline]    ──buffer──▶ [pending_sync]                │
│  [pending_sync] ──reconnect──▶ [synced]                 │
└─────────────────────────────────────────────────────────┘
```

**Online mode**: Changes are written locally first, then pushed to cloud within 500ms.

**Offline mode**: Changes are written to a local `pending_sync` queue. On reconnect, the queue is drained in order.

**Conflict resolution strategy**: Last-Write-Wins (LWW) based on `updated_at` timestamp, with vector clocks reserved for Phase 3. Conflicts are surfaced as a notification to the user in the system tray menu.

---

## SECTION 3: CORE FEATURES

### MVP Features (Phase 1)

| Feature | Priority | Notes |
|---|---|---|
| Desktop sticky notes | P0 | Native windows, always-on-top |
| Always-on-top mode | P0 | OS-level window flag |
| Task CRUD via SDK | P0 | create/update/delete/complete |
| Auto-start on boot | P0 | OS startup registration |
| Multi-monitor support | P0 | Detect active monitor, remember position per monitor |
| Reminder notifications | P0 | OS native notifications |
| Real-time updates | P0 | WebSocket push from web app to sticky |
| Offline support | P0 | Local SQLite; queue sync |
| Dark mode | P1 | Follow OS preference |
| Task completion tracking | P1 | Visual strikethrough + auto-dismiss |
| System tray icon | P0 | Show/hide all, view task list |
| Agent auto-updater | P1 | Background updates |

### Future Features (Phase 2+)

- AI task prioritization (LLM-based scoring)
- Voice reminders (TTS)
- Google Calendar / Outlook integration
- Slack integration (bi-directional)
- Jira / Linear webhook listener
- Microsoft Teams integration
- Shared team stickies (collaborative mode)
- Mobile companion app (view-only)

---

## SECTION 4: TECH STACK (with rationale)

### Frontend SDK

| Choice | Decision | Rationale |
|---|---|---|
| Language | TypeScript | Type safety; autocomplete for SDK consumers |
| React SDK | React 18 + hooks | Widest adoption; hooks API is ergonomic |
| State management | **Zustand** over Redux | Zustand is lighter, boilerplate-free, and sufficient for SDK-level state. Redux is overkill for a SDK with no complex derived state. |
| Styling (demo app) | **Tailwind + shadcn/ui** | shadcn/ui provides unstyled but accessible components; Tailwind keeps CSS atomic. Together they produce a solid demo UI with zero design debt. |
| Build | Vite + tsup | tsup for SDK bundle (CJS + ESM); Vite for example apps |

### Desktop Agent

| Choice | Decision | Rationale |
|---|---|---|
| Framework | **Tauri v2** | 10x smaller bundle; Rust backend; OS WebView; capability-based security |
| Backend language | Rust | Memory safety, performance, excellent async story with Tokio |
| HTTP server | **Axum** (Rust) | Ergonomic, async, tower-compatible; integrates naturally in Tauri |
| DB | SQLite via `sqlx` | Compile-time query validation; zero-config |
| Encryption | SQLCipher | Transparent AES-256 encryption of SQLite |

### Optional Cloud Backend

| Choice | Decision | Rationale |
|---|---|---|
| Runtime | Node.js 20 LTS | Familiar to most web developers; good TypeScript support |
| Framework | **NestJS** over Express | NestJS provides decorators, DI, module structure — essential for a production-grade API. Express is too bare for a team codebase. |
| Database | **PostgreSQL** | Relational, ACID, excellent JSON support, scales well |
| ORM | Prisma | Type-safe, excellent migrations, pairs well with NestJS |
| Cache | Redis | For WebSocket pub/sub across multiple cloud instances |
| Real-time | Socket.IO | Battle-tested; fallback transports; rooms |

### Communication

| Scenario | Protocol | Rationale |
|---|---|---|
| SDK → Agent (commands) | REST (HTTP) | Simple, debuggable, idempotent |
| SDK → Agent (events) | WebSocket | Persistent, low latency |
| Agent → Cloud | WebSocket + REST | WebSocket for real-time sync; REST for bulk operations |
| Internal (Tauri) | Tauri IPC commands | Type-safe Rust ↔ WebView bridge |

gRPC is explicitly **not used** on localhost. It adds proto compilation complexity with no meaningful performance gain over HTTP/2 + JSON at localhost speeds.

### Packaging & Distribution

| Tool | Purpose |
|---|---|
| `cargo-tauri` | Build and bundle the desktop agent |
| Tauri Updater | Auto-update via GitHub Releases (signed) |
| `changesets` | Changelog and npm versioning |
| GitHub Actions | CI/CD pipeline |
| Semantic Release | Automated release tagging |

### Monitoring

| Tool | Purpose |
|---|---|
| **Sentry** | Error tracking (both agent crash reports and cloud API errors) |
| **OpenTelemetry** | Distributed tracing for cloud backend |
| **Prometheus + Grafana** | Cloud API metrics |

---

## SECTION 5: OPEN SOURCE MONOREPO STRUCTURE

**Monorepo tool: Turborepo** over Nx.

Turborepo is simpler to configure for a mixed Rust + TypeScript monorepo. Nx's strength is Angular-ecosystem. For a Tauri + Node.js + React project, Turborepo's minimal config and caching model is ideal.

```
dtpf/
├── apps/
│   ├── desktop-agent/               # Tauri desktop agent (Rust + WebView)
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   ├── commands/        # Tauri command handlers
│   │   │   │   │   ├── task.rs
│   │   │   │   │   └── window.rs
│   │   │   │   ├── server/          # Axum HTTP/WS server
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── routes.rs
│   │   │   │   │   ├── websocket.rs
│   │   │   │   │   └── auth.rs
│   │   │   │   ├── db/              # SQLite / sqlx
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── migrations/
│   │   │   │   │   └── repository.rs
│   │   │   │   ├── tray/            # System tray
│   │   │   │   ├── window_manager/  # Native windows
│   │   │   │   └── sync/            # Cloud sync engine
│   │   │   ├── Cargo.toml
│   │   │   └── tauri.conf.json
│   │   └── ui/                      # Sticky note WebView UI (Vite + React)
│   │       ├── sticky-note/
│   │       └── tray-menu/
│   │
│   ├── cloud-api/                   # Optional NestJS cloud backend
│   │   ├── src/
│   │   │   ├── tasks/
│   │   │   ├── auth/
│   │   │   ├── sync/
│   │   │   └── main.ts
│   │   ├── prisma/
│   │   │   └── schema.prisma
│   │   └── package.json
│   │
│   └── docs/                        # Docusaurus documentation site
│       ├── docs/
│       └── package.json
│
├── packages/
│   ├── sdk-core/                    # Framework-agnostic SDK
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   ├── client.ts
│   │   │   ├── discovery.ts
│   │   │   ├── auth.ts
│   │   │   ├── ws.ts
│   │   │   └── types.ts
│   │   └── package.json
│   │
│   ├── sdk-react/                   # React hooks + context
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   ├── context/
│   │   │   │   └── DTPFProvider.tsx
│   │   │   └── hooks/
│   │   │       ├── useStickyTask.ts
│   │   │       ├── useTaskEvents.ts
│   │   │       └── useAgentStatus.ts
│   │   └── package.json
│   │
│   ├── sdk-vanilla/                 # Zero-dependency JS SDK
│   │   ├── src/
│   │   │   └── index.ts
│   │   └── package.json
│   │
│   ├── shared-types/                # Shared TS types (used by all packages)
│   │   ├── src/
│   │   │   └── index.ts
│   │   └── package.json
│   │
│   └── eslint-config/               # Shared lint config
│       └── index.js
│
├── examples/
│   ├── react-basic/                 # Minimal React integration example
│   ├── nextjs-tasks/                # Next.js task app with DTPF
│   ├── vanilla-js/                  # Vanilla JS example
│   └── vue-app/                     # Vue 3 example (community)
│
├── scripts/
│   ├── setup.sh                     # Dev environment setup
│   └── release.sh                   # Release automation
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       └── agent-build.yml
│
├── turbo.json
├── pnpm-workspace.yaml
└── README.md
```

---

## SECTION 6: SDK DESIGN

### 6.1 Core TypeScript Interfaces

```typescript
// packages/shared-types/src/index.ts

export type TaskStatus = 'active' | 'completed' | 'dismissed' | 'snoozed';
export type TaskPriority = 0 | 1 | 2 | 3; // low, normal, high, critical

export interface Task {
  id: string;
  title: string;
  body?: string;
  status: TaskStatus;
  priority: TaskPriority;
  color?: string;        // hex color for the sticky
  remindAt?: Date;
  metadata?: Record<string, unknown>;
}

export interface CreateTaskOptions {
  title: string;
  body?: string;
  priority?: TaskPriority;
  color?: string;
  remindAt?: Date;
  position?: { x: number; y: number };
  monitorId?: string;
  metadata?: Record<string, unknown>;
}

export interface UpdateTaskOptions {
  title?: string;
  body?: string;
  priority?: TaskPriority;
  color?: string;
  remindAt?: Date;
  metadata?: Record<string, unknown>;
}

export type TaskEvent =
  | { type: 'task:created';   task: Task }
  | { type: 'task:updated';   task: Task }
  | { type: 'task:completed'; taskId: string }
  | { type: 'task:dismissed'; taskId: string }
  | { type: 'task:reminder';  task: Task }
  | { type: 'agent:connected' }
  | { type: 'agent:disconnected' };

export interface AgentStatus {
  connected: boolean;
  version: string;
  platform: 'windows' | 'macos' | 'linux';
  taskCount: number;
}

export interface DTPFConfig {
  appId: string;           // Unique identifier for this web app
  appName: string;         // Human-readable name shown in agent UI
  token?: string;          // Auth token (generated on first registration)
  cloudSyncUrl?: string;   // Optional cloud backend URL
  timeout?: number;        // Request timeout in ms (default: 5000)
}
```

### 6.2 Core SDK API

```typescript
// packages/sdk-core/src/client.ts

export class DTPFClient {
  constructor(config: DTPFConfig) {}

  // Agent connectivity
  async connect(): Promise<void>
  async disconnect(): Promise<void>
  async getAgentStatus(): Promise<AgentStatus>

  // Task lifecycle
  async createStickyTask(options: CreateTaskOptions): Promise<Task>
  async updateStickyTask(taskId: string, options: UpdateTaskOptions): Promise<Task>
  async deleteStickyTask(taskId: string): Promise<void>
  async completeTask(taskId: string): Promise<void>
  async snoozeTask(taskId: string, until: Date): Promise<void>
  async getTask(taskId: string): Promise<Task | null>
  async listTasks(): Promise<Task[]>

  // Reminders
  async showReminder(taskId: string, message?: string): Promise<void>

  // Event subscription
  subscribeTaskEvents(handler: (event: TaskEvent) => void): Unsubscribe
  unsubscribeAll(): void
}

type Unsubscribe = () => void;
```

### 6.3 React SDK

```typescript
// packages/sdk-react/src/context/DTPFProvider.tsx

interface DTPFContextValue {
  client: DTPFClient;
  status: AgentStatus | null;
  isConnected: boolean;
}

export function DTPFProvider({ config, children }: {
  config: DTPFConfig;
  children: React.ReactNode;
}): JSX.Element

// packages/sdk-react/src/hooks/useStickyTask.ts
export function useStickyTask(): {
  createTask: (opts: CreateTaskOptions) => Promise<Task>;
  updateTask: (id: string, opts: UpdateTaskOptions) => Promise<Task>;
  deleteTask: (id: string) => Promise<void>;
  completeTask: (id: string) => Promise<void>;
  loading: boolean;
  error: Error | null;
}

// packages/sdk-react/src/hooks/useTaskEvents.ts
export function useTaskEvents(handler: (event: TaskEvent) => void): void

// packages/sdk-react/src/hooks/useAgentStatus.ts
export function useAgentStatus(): AgentStatus | null
```

### 6.4 Authentication Flow

```
First registration (one-time):
──────────────────────────────
1. Web app calls client.connect()
2. SDK opens system browser to http://localhost:7842/auth/register
   (Tauri opens a native dialog asking user to approve the app)
3. User approves → agent generates HMAC-SHA256 token tied to (appId + origin)
4. Token returned and stored in web app (localStorage) and agent DB
5. All subsequent requests include token in Authorization header

Subsequent connections:
────────────────────────
1. SDK sends token + appId in Authorization header
2. Agent validates HMAC signature
3. Connection established; no user interaction required
```

### 6.5 Local Discovery Mechanism

```typescript
// packages/sdk-core/src/discovery.ts

const WELL_KNOWN_PORTS = [7842, 7843, 7844]; // fallbacks
const LOCK_FILE_API = 'http://localhost:7842/health';

async function discoverAgent(): Promise<string> {
  // Strategy 1: Try well-known port
  for (const port of WELL_KNOWN_PORTS) {
    try {
      const res = await fetch(`http://localhost:${port}/health`, {
        signal: AbortSignal.timeout(500)
      });
      if (res.ok) return `http://localhost:${port}`;
    } catch {}
  }
  // Strategy 2: Ask user to open agent (show install prompt)
  throw new AgentNotFoundError('DTPF agent not running. Please install and start it.');
}
```

---

## SECTION 7: DESKTOP AGENT DESIGN

### 7.1 System Tray App

The system tray icon is the user-facing control center of the agent.

```
Tray Icon Menu:
├── [DTPF Logo] Desktop Task Presence
├── ─────────────────────────────────
├── 📋 Active Tasks (3)
│   ├── ✓ Fix login bug [Acme App]
│   ├── ⏰ Deploy to staging [Acme App]
│   └── 🔴 Critical: DB backup [Ops Dashboard]
├── ─────────────────────────────────
├── 👁  Show All Stickies
├── 🙈 Hide All Stickies
├── ─────────────────────────────────
├── 🔌 Connected Apps (2)
│   ├── Acme Task Manager
│   └── Ops Dashboard
├── ─────────────────────────────────
├── ⚙️  Preferences
├── 🔄 Check for Updates
└── ✖  Quit
```

### 7.2 Native Desktop Windows (Sticky Notes)

Each sticky note is a Tauri WebviewWindow with the following properties:

```rust
// src-tauri/src/window_manager/mod.rs

pub fn create_sticky_window(task: &Task, app: &AppHandle) -> Result<WebviewWindow> {
    let window = WebviewWindowBuilder::new(
        app,
        &format!("sticky-{}", task.id),
        WebviewUrl::App(format!("sticky-note/index.html?taskId={}", task.id).into())
    )
    .title("")
    .inner_size(280.0, 200.0)
    .decorations(false)          // no OS title bar
    .always_on_top(true)         // float above all apps
    .skip_taskbar(true)          // don't appear in taskbar
    .transparent(true)           // rounded corners via CSS
    .resizable(true)
    .position(task.position_x as f64, task.position_y as f64)
    .build()?;

    Ok(window)
}
```

Sticky note UI (WebView content):

- Rounded card design with the task color as background
- Drag handle (allows repositioning; position saved to DB on drop)
- Task title (bold), body (text), priority indicator, source app badge
- Dismiss (✕) and Complete (✓) buttons
- Countdown timer if `remindAt` is set
- Real-time updates via Tauri event system (no polling)

### 7.3 Window Persistence

On agent startup:

```rust
async fn restore_windows(db: &Database, app: &AppHandle) -> Result<()> {
    let active_tasks = db.list_active_tasks().await?;
    for task in active_tasks {
        create_sticky_window(&task, app)?;
    }
    Ok(())
}
```

Window positions are saved to SQLite on every drag-end event:

```rust
#[tauri::command]
async fn save_window_position(
    task_id: String, x: i32, y: i32,
    db: State<'_, Database>
) -> Result<(), String> {
    db.update_position(&task_id, x, y).await.map_err(|e| e.to_string())
}
```

### 7.4 Startup Registration

**Windows**: Registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

```rust
#[cfg(target_os = "windows")]
fn register_startup() {
    use winreg::enums::*;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_SET_VALUE
    ).unwrap();
    run.set_value("DTPF", &std::env::current_exe().unwrap().to_str().unwrap()).unwrap();
}
```

**macOS**: LaunchAgent plist at `~/Library/LaunchAgents/com.dtpf.agent.plist`

**Linux**: systemd user service at `~/.config/systemd/user/dtpf-agent.service`

### 7.5 Multi-Monitor Support

```rust
// Monitor detection at window creation
fn get_primary_monitor(app: &AppHandle) -> Monitor {
    app.primary_monitor()
       .expect("no monitor found")
       .expect("no primary monitor")
}

fn get_monitor_for_position(app: &AppHandle, x: i32, y: i32) -> Option<Monitor> {
    app.available_monitors().ok()?.into_iter().flatten().find(|m| {
        let pos = m.position();
        let size = m.size();
        x >= pos.x && x <= pos.x + size.width as i32 &&
        y >= pos.y && y <= pos.y + size.height as i32
    })
}
```

Sticky notes remember which monitor they were on (by monitor `name`). On restore, the agent checks if the monitor is still available; if not, it falls back to the primary monitor.

### 7.6 Platform-Specific Notes

| Feature | Windows | macOS | Linux |
|---|---|---|---|
| Always-on-top | `SetWindowPos` HWND_TOPMOST | NSWindow level | `_NET_WM_STATE_ABOVE` |
| Startup | Registry HKCU Run | LaunchAgent plist | systemd user service |
| Tray | System tray (Win32) | NSStatusBar | libappindicator / GTK |
| Notifications | Windows Notifications API | UNUserNotificationCenter | libnotify |
| Transparency | DwmExtendFrameIntoClientArea | NSWindow backgroundColor = .clear | compositor (Picom) |

---

## SECTION 8: SECURITY

### 8.1 Threat Model

```
Threat: A malicious website creates stickies on behalf of the user without consent
Mitigation: Token-based authentication + explicit user approval on first registration

Threat: Another local process intercepts/modifies agent traffic
Mitigation: HMAC-signed tokens; localhost-only binding; origin validation

Threat: A rogue web app spams the agent with thousands of tasks
Mitigation: Rate limiting per app_id (100 req/min default); configurable per app

Threat: Agent database contains sensitive task data
Mitigation: SQLCipher AES-256 encryption at rest; key derived from OS keychain

Threat: Attacker reads the auth token from localStorage
Mitigation: Tokens are scoped to (appId + origin); stolen token is useless on different origin

Threat: Malicious update payload
Mitigation: Code-signed binaries; Tauri updater validates signature before applying
```

### 8.2 Localhost API Security

```rust
// src-tauri/src/server/auth.rs

pub struct AuthMiddleware;

impl<S> tower::Layer<S> for AuthMiddleware {
    // Validates:
    // 1. Authorization: Bearer <hmac_token>
    // 2. Origin header matches registered origin for this appId
    // 3. X-DTPF-App-ID header present and registered
    // 4. Request is from 127.0.0.1 or ::1 (localhost only)
}
```

**CORS policy**: Agent only accepts requests from registered origins. All other origins receive 403.

**Binding**: Agent binds exclusively to `127.0.0.1:7842`, never `0.0.0.0`.

### 8.3 Token Design

```
Token = HMAC-SHA256(appId + ":" + origin + ":" + created_at, secret_key)
secret_key = 32-byte random key stored in OS keychain (Keychain on macOS, Credential Manager on Windows, libsecret on Linux)
```

Tokens are long-lived (no expiry by default) but revocable by the user from the tray menu.

### 8.4 Rate Limiting

```rust
// Per-app sliding window rate limiter
// Default: 100 requests/minute per app_id
// Burst: 20 requests/second
// Configurable in preferences
```

### 8.5 Encrypted Local Storage

SQLCipher is initialized with a key derived from the OS keychain:

```rust
fn open_db(path: &Path) -> Result<SqlitePool> {
    let key = keychain::get_or_create_key("dtpf-db-key")?;
    SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
                .pragma("key", key)
                .pragma("cipher_page_size", "4096")
        ).await
}
```

---

## SECTION 9: IMPLEMENTATION ROADMAP

### Phase 1: MVP — 2 Weeks

**Goal**: Core sticky note creation from a React web app on a single machine.

| Week | Tasks | Complexity |
|---|---|---|
| Week 1 | Monorepo setup; Tauri agent scaffold; SQLite schema; Axum HTTP server; basic `createStickyTask` endpoint | Medium |
| Week 1 | Sticky note WebView UI; always-on-top; drag + position save; system tray with task list | Medium |
| Week 2 | `sdk-core` npm package; `sdk-react` hooks; local discovery; auth token flow | Medium |
| Week 2 | Auto-start on boot (Windows + macOS); basic offline support; real-time WebSocket events | Medium-High |
| Week 2 | React demo app; packaging; basic CI pipeline | Low |

**End state**: A developer can `npm install @dtpf/sdk-react`, wrap their app in `<DTPFProvider>`, and call `createStickyTask()` to create a native sticky on their desktop.

### Phase 2: Public Beta — 4 Weeks

- Linux support
- Multi-monitor aware window placement
- Task update + completion lifecycle
- Dark mode sticky notes
- Reminder notifications (OS native)
- Sentry integration
- Auto-updater
- `sdk-vanilla` and Angular adapter
- Comprehensive test suite (Rust unit tests + Playwright integration tests)
- Documentation site (Docusaurus)

### Phase 3: Open Source Launch — 4 Weeks

- Cloud sync backend (NestJS + PostgreSQL)
- Cross-device sync
- Conflict resolution UI
- GitHub-ready: CONTRIBUTING.md, issue templates, PR templates, CoC
- NPM publish (`@dtpf/sdk-core`, `@dtpf/sdk-react`, `@dtpf/sdk-vanilla`)
- GitHub release with signed binaries for Win/Mac/Linux
- Product Hunt launch

### Phase 4: Enterprise Edition — Ongoing

- Centralized team task management
- SSO (SAML, OIDC)
- Admin console (who created what stickies, audit log)
- Custom branding (sticky note themes per org)
- SLA-based sticky escalation
- Slack / Teams / Jira webhook integration
- Enterprise support SLA

**Development Effort Estimate**

| Phase | Effort (eng-weeks) | Team |
|---|---|---|
| MVP | 2 weeks × 1 engineer | Solo feasible |
| Public Beta | 4 weeks × 2 engineers | Small team |
| OSS Launch | 4 weeks × 2–3 engineers | Small team |
| Enterprise | Ongoing × 3–5 engineers | Startup team |

---

## SECTION 10: BUSINESS POTENTIAL

### Open Source Viability

DTPF solves a genuine, unsolved problem (web → desktop presence bridge). The framework pattern (SDK + local agent) is well-proven: Stripe, Plaid, Segment all built massive businesses on developer SDKs.

The open-source core drives adoption; the enterprise layer drives revenue. This is the "open core" model (HashiCorp, Posthog, Cal.com).

### Enterprise Demand

Strong demand signals:

- **Field operations tools**: Logistics, healthcare, manufacturing workers need persistent overlays
- **Financial dashboards**: Trading desks need always-visible alerts
- **DevOps dashboards**: On-call engineers need desktop-level incident visibility
- **Education platforms**: Remote learning tools need presence beyond browser tabs

### Monetization Paths

| Model | Description |
|---|---|
| **Open Core** | Core framework free; enterprise features (SSO, team management, admin, SLA) paid |
| **Cloud Sync SaaS** | Free for self-hosted; $5–15/user/month for managed cloud sync |
| **Enterprise License** | Annual license for large teams; custom integrations |
| **Priority Support** | Paid support contracts for production deployments |

### Adoption Strategy

1. **Developer-first**: Publish SDK to npm; write tutorials for popular platforms (Next.js, React)
2. **Community**: Discord, GitHub Discussions, monthly changelog
3. **Integrations**: Build Jira and Slack plugins to drive organic adoption from those ecosystems
4. **Content marketing**: "Building desktop presence for web apps" technical blog posts
5. **Product Hunt**: Launch publicly with a polished demo video
6. **Open source PR**: Contribute talks to React Conf, Node.js Summit

---

## SECTION 11: CURSOR AI EXECUTION PLAN

### Epic 1: Monorepo Foundation

**Feature 1.1: Initialize monorepo**

```
Cursor Prompt:
"Create a new Turborepo monorepo with pnpm workspaces.
Structure:
- apps/desktop-agent (Tauri v2 app, to be initialized)
- apps/cloud-api (NestJS, to be initialized)
- packages/sdk-core (TypeScript library)
- packages/sdk-react (React library)
- packages/shared-types (shared TypeScript types)
- packages/eslint-config (shared ESLint config)

Configure turbo.json with pipelines for: build, dev, lint, test.
Set up shared tsconfig.base.json.
Set up pnpm-workspace.yaml.
Initialize git with .gitignore for Node, Rust, and editors.
Do not initialize the Tauri or NestJS apps yet."
```

**Feature 1.2: Shared types package**

```
Cursor Prompt:
"In packages/shared-types/src/index.ts, implement the following TypeScript interfaces
exactly as specified:

[paste the full types from Section 6.1]

Set up the package.json with:
  name: @dtpf/shared-types
  exports: { '.': { import: './dist/index.js', require: './dist/index.cjs' } }
Use tsup for building. Make sure it compiles to both ESM and CJS."
```

---

### Epic 2: Desktop Agent — Rust Backend

**Feature 2.1: Initialize Tauri v2 project**

```
Cursor Prompt:
"In apps/desktop-agent, initialize a Tauri v2 project using:
  cargo tauri init
Configure tauri.conf.json:
  - productName: DTPF Agent
  - identifier: com.dtpf.agent
  - windows: single main window, hidden on startup
  - systemTray: enabled with icon

Set up the Rust project structure:
  src-tauri/src/
    main.rs           (app bootstrap, tray setup)
    commands/
      task.rs         (Tauri commands for task CRUD)
      window.rs       (window management commands)
    server/
      mod.rs
      routes.rs       (Axum router)
      websocket.rs    (WS handler)
      auth.rs         (HMAC middleware)
    db/
      mod.rs
      migrations/     (sqlx migrations)
      repository.rs   (DB query functions)
    tray/mod.rs
    window_manager/mod.rs
    sync/mod.rs

Add Cargo dependencies:
  tauri = { version = '2', features = ['tray-icon', 'shell-open'] }
  axum = '0.7'
  tokio = { version = '1', features = ['full'] }
  sqlx = { version = '0.7', features = ['sqlite', 'runtime-tokio', 'chrono'] }
  serde = { version = '1', features = ['derive'] }
  serde_json = '1'
  hmac = '0.12'
  sha2 = '0.10'
  uuid = { version = '1', features = ['v4'] }"
```

**Feature 2.2: SQLite schema and migrations**

```
Cursor Prompt:
"In apps/desktop-agent/src-tauri/src/db/, implement:

1. migrations/0001_initial.sql with the schema from [Section 2.5]

2. repository.rs with async functions:
   - create_task(task: &NewTask) -> Result<Task>
   - get_task(id: &str) -> Result<Option<Task>>
   - list_active_tasks() -> Result<Vec<Task>>
   - update_task(id: &str, update: &TaskUpdate) -> Result<Task>
   - delete_task(id: &str) -> Result<()>
   - update_position(id: &str, x: i32, y: i32) -> Result<()>
   - register_app(app: &AppRegistration) -> Result<()>
   - get_app_by_id(app_id: &str) -> Result<Option<AppRegistration>>

3. mod.rs that initializes the SQLite connection pool using sqlx::SqlitePool,
   runs migrations on startup, and exports a Database struct wrapping the pool.

Use sqlx::query_as! macros for compile-time query validation."
```

**Feature 2.3: Axum HTTP server**

```
Cursor Prompt:
"In apps/desktop-agent/src-tauri/src/server/, implement an Axum web server that:

1. Binds to 127.0.0.1:7842
2. Implements routes:
   GET  /health               → { version, status, taskCount }
   POST /auth/register        → register app, return token
   GET  /tasks                → list active tasks
   POST /tasks                → create task
   GET  /tasks/:id            → get task
   PUT  /tasks/:id            → update task
   DELETE /tasks/:id          → delete task
   POST /tasks/:id/complete   → complete task
   POST /tasks/:id/snooze     → snooze task
   GET  /ws                   → WebSocket upgrade

3. Auth middleware that validates:
   - Request from 127.0.0.1 only
   - Authorization: Bearer <token> header
   - X-DTPF-App-ID header matches token

4. Rate limiting: 100 requests/minute per app_id using tower-governor

Start the server in a separate Tokio task from main.rs."
```

**Feature 2.4: Window manager**

```
Cursor Prompt:
"In apps/desktop-agent/src-tauri/src/window_manager/mod.rs, implement:

1. create_sticky_window(task: &Task, app: &AppHandle) -> Result<WebviewWindow>
   - decorations: false
   - always_on_top: true
   - skip_taskbar: true
   - transparent: true
   - size: 280x200
   - position: from task.position_x/y, or default (100 + index*20, 100 + index*20)

2. destroy_sticky_window(task_id: &str, app: &AppHandle) -> Result<()>

3. restore_all_windows(db: &Database, app: &AppHandle) -> Result<()>
   Called on agent startup to recreate windows for all active tasks.

4. update_sticky_content(task_id: &str, task: &Task, app: &AppHandle)
   Uses app.emit_to() to send updated task data to the sticky note WebView.

5. save_position_on_drop(task_id: &str, x: i32, y: i32, db: &Database)
   Called from a Tauri command when the user drags and drops a sticky note."
```

---

### Epic 3: Sticky Note UI

**Feature 3.1: Sticky note WebView**

```
Cursor Prompt:
"In apps/desktop-agent/ui/sticky-note/, create a Vite + React app that renders a sticky note.

Design requirements:
- Reads taskId from URL query param
- Uses Tauri invoke() to fetch task data: invoke('get_task', { taskId })
- Listens for task:updated events from Tauri: listen('task:updated', handler)
- Renders: drag handle bar (top strip), task title (bold), body text, 
  priority badge (colored dot), source app name (small, bottom left),
  Complete button (✓), Dismiss button (✗)
- Drag to reposition: on dragend, calls invoke('save_window_position', {taskId, x, y})
- Sticky background color from task.color (default #FFE066)
- Rounded corners (border-radius: 12px), subtle shadow
- Dark mode: follows prefers-color-scheme
- Transparent window background, card is the colored element

Style with Tailwind CSS. Keep the component under 150 lines."
```

---

### Epic 4: SDK

**Feature 4.1: sdk-core**

```
Cursor Prompt:
"In packages/sdk-core/src/, implement the DTPFClient class as specified in Section 6.2.

Implementation details:
- discovery.ts: probe localhost ports [7842, 7843, 7844] with 500ms timeout each
- http-client.ts: fetch-based REST client; adds Authorization + X-DTPF-App-ID headers
- ws-client.ts: native WebSocket; auto-reconnects with exponential backoff (max 30s)
- auth.ts: stores token in localStorage under key dtpf_token_{appId}; 
  sends to /auth/register on first connect
- All methods return Promise; throw DTPFError on failure with error codes:
  AGENT_NOT_FOUND, AUTH_FAILED, RATE_LIMITED, TASK_NOT_FOUND, NETWORK_ERROR

Export DTPFClient and all types.
Build target: ESM + CJS via tsup."
```

**Feature 4.2: sdk-react**

```
Cursor Prompt:
"In packages/sdk-react/src/, implement:

1. DTPFProvider: wraps DTPFClient, initializes on mount, calls connect(),
   provides context with { client, status, isConnected }

2. useStickyTask(): returns { createTask, updateTask, deleteTask, completeTask, loading, error }
   All functions are async wrappers around client methods.
   loading is true while any operation is in-flight.
   error is the last error thrown (null on success).

3. useTaskEvents(handler): subscribes to client events on mount,
   unsubscribes on unmount.

4. useAgentStatus(): polls client.getAgentStatus() every 30s, 
   returns current AgentStatus or null.

Use React 18. No class components. All hooks."
```

---

### Epic 5: Auth & Security

**Feature 5.1: Token registration flow**

```
Cursor Prompt:
"In apps/desktop-agent/src-tauri/src/server/auth.rs, implement:

1. POST /auth/register handler:
   - Receives: { appId, appName, origin }
   - Checks if app already registered (return existing token)
   - If new: shows a Tauri native dialog asking user:
     '{appName} ({origin}) wants to create desktop sticky notes. Allow?'
   - On approval: generates HMAC-SHA256 token, stores in DB, returns token
   - On denial: returns 403

2. Auth middleware (tower::Layer):
   - Extracts Authorization: Bearer <token>
   - Extracts X-DTPF-App-ID header
   - Validates token against DB
   - Validates Origin header matches registered origin
   - Validates request IP is 127.0.0.1
   - Returns 401 on any failure

3. Rate limiter: sliding window, 100 req/min per app_id,
   returns 429 with Retry-After header on breach."
```

---

### Epic 6: CI/CD and Packaging

**Feature 6.1: GitHub Actions CI**

```
Cursor Prompt:
"Create .github/workflows/ci.yml that:

1. Triggers on: push to main, pull requests
2. Jobs:
   a. lint-and-test (ubuntu-latest):
      - pnpm install
      - turbo run lint
      - turbo run test
   b. build-agent (matrix: windows-latest, macos-latest, ubuntu-latest):
      - Install Rust stable + Tauri CLI deps per platform
      - pnpm install
      - cargo test (in apps/desktop-agent/src-tauri)
      - cargo tauri build
   c. build-sdk:
      - pnpm install
      - turbo run build --filter=@dtpf/sdk-*

Use pnpm caching. Use tauri-action from tauri-apps/tauri-action@v0."
```

**Feature 6.2: Release workflow**

```
Cursor Prompt:
"Create .github/workflows/release.yml that:

1. Triggers on: push of tags matching v*.*.*
2. Uses changesets/action to publish npm packages:
   - @dtpf/sdk-core
   - @dtpf/sdk-react
   - @dtpf/sdk-vanilla
   - @dtpf/shared-types
3. Uses tauri-action to build and upload signed binaries for:
   - Windows (.msi, .exe)
   - macOS (.dmg, .app.tar.gz)
   - Linux (.AppImage, .deb)
4. Creates GitHub Release with:
   - Changelog (from changesets)
   - All binary artifacts attached
   - NPM publish links

Requires secrets: TAURI_PRIVATE_KEY, TAURI_KEY_PASSWORD, NPM_TOKEN."
```

---

### Epic 7: Demo Application

**Feature 7.1: Next.js demo app**

```
Cursor Prompt:
"In examples/nextjs-tasks/, create a Next.js 14 app (App Router) that demonstrates DTPF.

Pages:
1. / — Dashboard with list of tasks; button to 'Accept Task' which calls createStickyTask()
2. /tasks/[id] — Task detail with 'Mark Complete' and 'Update' buttons

Use @dtpf/sdk-react. Wrap in DTPFProvider in layout.tsx.

When createStickyTask succeeds, show a toast: 'Sticky note created on your desktop!'
When agent is not connected, show a banner: 'DTPF Agent not running. Download it here.'

Use Tailwind CSS + shadcn/ui. Keep it realistic: use a plausible task list
(e.g., 'Fix login bug', 'Deploy to staging', 'Review PR #42').
Add a status indicator in the top-right corner showing agent connection status."
```

---

### Epic 8: Documentation

**Feature 8.1: README**

```
Cursor Prompt:
"Write a comprehensive README.md for the DTPF monorepo root.

Sections:
1. What is DTPF? (2 sentences)
2. Demo GIF placeholder [screenshot]
3. Quick Start (SDK install + DTPFProvider + createStickyTask() — under 20 lines of code)
4. How it works (architecture diagram in ASCII from Section 2.1)
5. Installation (SDK: npm; Agent: download links per platform)
6. API Reference (link to docs site)
7. Examples (link to examples/)
8. Contributing (link to CONTRIBUTING.md)
9. License (MIT)

Tone: developer-friendly, direct, no marketing fluff.
Every code block must be copy-pasteable and correct."
```

---

*End of Desktop Task Presence Framework — Architecture & Technical Plan v1.0*

*Document generated for engineering team handoff. All section references are internally consistent.*
*Estimated total implementation: 10–14 engineer-weeks for MVP through OSS launch.*
