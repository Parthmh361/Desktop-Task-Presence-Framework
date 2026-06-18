CREATE TABLE IF NOT EXISTS tasks (
  id            TEXT PRIMARY KEY,
  source_app_id TEXT NOT NULL,
  title         TEXT NOT NULL,
  body          TEXT,
  status        TEXT NOT NULL DEFAULT 'active',
  priority      INTEGER NOT NULL DEFAULT 0,
  color         TEXT NOT NULL DEFAULT '#FFE066',
  position_x    INTEGER,
  position_y    INTEGER,
  monitor_id    TEXT,
  remind_at     INTEGER,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  synced_at     INTEGER,
  metadata      TEXT
);

CREATE TABLE IF NOT EXISTS app_registrations (
  app_id        TEXT PRIMARY KEY,
  app_name      TEXT NOT NULL,
  origin        TEXT NOT NULL,
  token         TEXT NOT NULL,
  created_at    INTEGER NOT NULL,
  last_seen_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_source_app ON tasks(source_app_id);
