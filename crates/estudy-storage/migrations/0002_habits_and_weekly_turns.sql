-- ============================================================================
-- MIGRAÇÃO v2: weekly_turns em projects + tabelas de Hábitos (habits, habit_logs)
-- ============================================================================

CREATE TABLE projects_v2 (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    color_token TEXT NOT NULL DEFAULT 'accent',
    status      TEXT NOT NULL DEFAULT 'planning'
                CHECK (status IN ('planning', 'in_progress', 'paused', 'done')),
    framework   TEXT NOT NULL DEFAULT 'kanban'
                CHECK (framework IN ('kanban', 'milestones', 'eisenhower', 'weekly_focus', 'weekly_turns')),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    extra_json  TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

INSERT INTO projects_v2 (
    id, title, description, color_token, status, framework,
    sort_order, extra_json, created_at, updated_at, deleted_at
)
SELECT
    id, title, description, color_token, status, framework,
    sort_order, extra_json, created_at, updated_at, deleted_at
FROM projects;

DROP TABLE projects;
ALTER TABLE projects_v2 RENAME TO projects;

CREATE INDEX IF NOT EXISTS idx_projects_active
    ON projects(status, sort_order ASC) WHERE deleted_at IS NULL;

-- ============================================================================
-- HÁBITOS E LOGS DE CONSTÂNCIA DIÁRIA
-- ============================================================================
CREATE TABLE IF NOT EXISTS habits (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    subtitle    TEXT NOT NULL DEFAULT '',
    color_index INTEGER NOT NULL DEFAULT 0 CHECK (color_index >= 0),
    weekly_goal INTEGER NOT NULL DEFAULT 7 CHECK (weekly_goal BETWEEN 1 AND 7),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_habits_active
    ON habits(sort_order ASC, created_at ASC) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS habit_logs (
    habit_id   TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    log_date   TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (habit_id, log_date)
);

CREATE INDEX IF NOT EXISTS idx_habit_logs_date
    ON habit_logs(log_date DESC, habit_id);
