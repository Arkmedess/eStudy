-- Schema baseline v1 do eStudy (v0.0.1)
-- Organizado em 3 camadas:
--   Camada 1: Dados do usuário (UUIDv7, extra_json, deleted_at)
--   Camada 2: Economia HMAC, gamificação, personalização e auditoria MCP
--   Camada 3: Controle de migrações e cache reconstruível

-- ============================================================================
-- CAMADA 3: CONTROLE DE MIGRAÇÕES E CACHE RECONSTRUÍVEL
-- ============================================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
    version          INTEGER PRIMARY KEY,
    name             TEXT NOT NULL,
    checksum_sha256  TEXT NOT NULL,
    applied_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS asset_cache (
    cache_key        TEXT PRIMARY KEY,
    file_path        TEXT NOT NULL,
    mime_type        TEXT NOT NULL,
    size_bytes       INTEGER NOT NULL CHECK (size_bytes >= 0),
    expires_at       TEXT,
    last_accessed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_asset_cache_lru
    ON asset_cache(last_accessed_at ASC);

-- ============================================================================
-- CAMADA 1: PERFIL LOCAL E BEM-ESTAR
-- ============================================================================
CREATE TABLE IF NOT EXISTS local_profiles (
    id                        TEXT PRIMARY KEY,
    display_name              TEXT NOT NULL,
    avatar_path               TEXT,
    daily_focus_goal_minutes  INTEGER NOT NULL DEFAULT 120 CHECK (daily_focus_goal_minutes > 0),
    extra_json                TEXT NOT NULL DEFAULT '{}',
    created_at                TEXT NOT NULL,
    updated_at                TEXT NOT NULL,
    deleted_at                TEXT
);

CREATE TABLE IF NOT EXISTS mood_logs (
    id          TEXT PRIMARY KEY,
    profile_id  TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    log_date    TEXT NOT NULL,
    mood_score  INTEGER NOT NULL CHECK (mood_score BETWEEN 1 AND 5),
    mood_label  TEXT NOT NULL,
    note        TEXT NOT NULL DEFAULT '',
    extra_json  TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    UNIQUE (profile_id, log_date)
);
CREATE INDEX IF NOT EXISTS idx_mood_logs_date
    ON mood_logs(profile_id, log_date DESC) WHERE deleted_at IS NULL;

-- ============================================================================
-- CAMADA 1: PROJETOS, TAREFAS, TAGS E TEMPO UNIFICADO
-- ============================================================================
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    color_token TEXT NOT NULL DEFAULT 'accent',
    status      TEXT NOT NULL DEFAULT 'planning'
                CHECK (status IN ('planning', 'in_progress', 'paused', 'done')),
    framework   TEXT NOT NULL DEFAULT 'kanban'
                CHECK (framework IN ('kanban', 'milestones', 'eisenhower', 'weekly_focus')),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    extra_json  TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX IF NOT EXISTS idx_projects_active
    ON projects(status, sort_order ASC) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS tasks (
    id                  TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    description_md      TEXT NOT NULL DEFAULT '',
    status              TEXT NOT NULL DEFAULT 'todo'
                        CHECK (status IN ('backlog', 'todo', 'in_progress', 'done')),
    priority            TEXT NOT NULL DEFAULT 'medium'
                        CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    eisenhower_quadrant TEXT
                        CHECK (eisenhower_quadrant IN ('do', 'schedule', 'delegate', 'eliminate') OR eisenhower_quadrant IS NULL),
    pinned_focus        INTEGER NOT NULL DEFAULT 0 CHECK (pinned_focus IN (0, 1)),
    start_at            TEXT,
    due_at              TEXT,
    recurrence_json     TEXT,
    estimated_pomos     INTEGER NOT NULL DEFAULT 1 CHECK (estimated_pomos >= 0),
    completed_pomos     INTEGER NOT NULL DEFAULT 0 CHECK (completed_pomos >= 0),
    sort_order          INTEGER NOT NULL DEFAULT 0,
    completed_at        TEXT,
    extra_json          TEXT NOT NULL DEFAULT '{}',
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT
);
CREATE INDEX IF NOT EXISTS idx_tasks_board
    ON tasks(status, due_at ASC, sort_order ASC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_focus_pin
    ON tasks(pinned_focus, sort_order ASC) WHERE pinned_focus = 1 AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS project_tasks (
    project_id     TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    task_id        TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    milestone_name TEXT,
    sort_order     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (project_id, task_id)
);
CREATE INDEX IF NOT EXISTS idx_project_tasks_task
    ON project_tasks(task_id);

CREATE TABLE IF NOT EXISTS task_checklist_items (
    id         TEXT PRIMARY KEY,
    task_id    TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    done       INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_task_checklist_task
    ON task_checklist_items(task_id, sort_order ASC);

CREATE TABLE IF NOT EXISTS tags (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE COLLATE NOCASE,
    color_token TEXT NOT NULL DEFAULT 'accent',
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entity_tags (
    tag_id      TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('project', 'task', 'event', 'subject', 'book', 'note')),
    entity_id   TEXT NOT NULL,
    PRIMARY KEY (tag_id, entity_type, entity_id)
);
CREATE INDEX IF NOT EXISTS idx_entity_tags_lookup
    ON entity_tags(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    project_id      TEXT REFERENCES projects(id) ON DELETE SET NULL,
    starts_at       TEXT NOT NULL,
    ends_at         TEXT NOT NULL,
    all_day         INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    recurrence_json TEXT,
    color_token     TEXT NOT NULL DEFAULT 'accent',
    extra_json      TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);
CREATE INDEX IF NOT EXISTS idx_calendar_events_range
    ON calendar_events(starts_at ASC, ends_at ASC) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS task_templates (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL UNIQUE,
    category     TEXT NOT NULL CHECK (category IN ('study', 'reading', 'project', 'general')),
    payload_json TEXT NOT NULL,
    extra_json   TEXT NOT NULL DEFAULT '{}',
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    deleted_at   TEXT
);

-- ============================================================================
-- CAMADA 1: ESTUDOS (CICLO, TÓPICOS, SESSÕES, SRS/FSRS, FLASHCARDS E ERROS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS study_subjects (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    color_token     TEXT NOT NULL DEFAULT 'accent',
    cycle_position  INTEGER NOT NULL DEFAULT 0,
    target_minutes  INTEGER NOT NULL DEFAULT 60 CHECK (target_minutes > 0),
    active_in_cycle INTEGER NOT NULL DEFAULT 1 CHECK (active_in_cycle IN (0, 1)),
    extra_json      TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);

CREATE TABLE IF NOT EXISTS study_topics (
    id            TEXT PRIMARY KEY,
    subject_id    TEXT NOT NULL REFERENCES study_subjects(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    mastery_level REAL NOT NULL DEFAULT 0.0,
    extra_json    TEXT NOT NULL DEFAULT '{}',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT,
    UNIQUE (subject_id, title)
);

CREATE TABLE IF NOT EXISTS spaced_reviews (
    id            TEXT PRIMARY KEY,
    subject_id    TEXT NOT NULL REFERENCES study_subjects(id) ON DELETE CASCADE,
    topic_id      TEXT REFERENCES study_topics(id) ON DELETE SET NULL,
    topic_title   TEXT NOT NULL,
    algorithm     TEXT NOT NULL DEFAULT 'fixed' CHECK (algorithm IN ('fixed', 'fsrs')),
    interval_days INTEGER NOT NULL CHECK (interval_days > 0),
    ease_factor   REAL NOT NULL DEFAULT 2.5,
    stability     REAL NOT NULL DEFAULT 1.0,
    difficulty    REAL NOT NULL DEFAULT 5.0,
    due_date      TEXT NOT NULL,
    completed_at  TEXT,
    extra_json    TEXT NOT NULL DEFAULT '{}',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
);
CREATE INDEX IF NOT EXISTS idx_spaced_reviews_due
    ON spaced_reviews(due_date ASC) WHERE completed_at IS NULL AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS study_planner_blocks (
    id              TEXT PRIMARY KEY,
    subject_id      TEXT NOT NULL REFERENCES study_subjects(id) ON DELETE CASCADE,
    topic_id        TEXT REFERENCES study_topics(id) ON DELETE SET NULL,
    weekday         INTEGER NOT NULL CHECK (weekday BETWEEN 0 AND 6),
    modality        TEXT NOT NULL CHECK (modality IN ('theory', 'questions', 'review')),
    planned_minutes INTEGER NOT NULL CHECK (planned_minutes > 0),
    extra_json      TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);

CREATE TABLE IF NOT EXISTS flashcards (
    id            TEXT PRIMARY KEY,
    subject_id    TEXT NOT NULL REFERENCES study_subjects(id) ON DELETE CASCADE,
    topic_id      TEXT REFERENCES study_topics(id) ON DELETE SET NULL,
    front_md      TEXT NOT NULL,
    back_md       TEXT NOT NULL,
    due_at        TEXT NOT NULL,
    interval_days INTEGER NOT NULL DEFAULT 1 CHECK (interval_days >= 0),
    ease_factor   REAL NOT NULL DEFAULT 2.5,
    reps          INTEGER NOT NULL DEFAULT 0,
    lapses        INTEGER NOT NULL DEFAULT 0,
    last_rating   TEXT CHECK (last_rating IN ('again', 'hard', 'good', 'easy') OR last_rating IS NULL),
    extra_json    TEXT NOT NULL DEFAULT '{}',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
);
CREATE INDEX IF NOT EXISTS idx_flashcards_due
    ON flashcards(subject_id, due_at ASC) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS error_notebook (
    id               TEXT PRIMARY KEY,
    subject_id       TEXT NOT NULL REFERENCES study_subjects(id) ON DELETE CASCADE,
    topic_id         TEXT REFERENCES study_topics(id) ON DELETE SET NULL,
    question_summary TEXT NOT NULL,
    mistake_reason   TEXT NOT NULL,
    correction_md    TEXT NOT NULL,
    extra_json       TEXT NOT NULL DEFAULT '{}',
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT
);
CREATE INDEX IF NOT EXISTS idx_error_notebook_subject
    ON error_notebook(subject_id, created_at DESC) WHERE deleted_at IS NULL;

-- ============================================================================
-- CAMADA 1: ESTANTE DE LEITURAS (ESTILO MARATONA.APP)
-- ============================================================================
CREATE TABLE IF NOT EXISTS books (
    id                TEXT PRIMARY KEY,
    title             TEXT NOT NULL,
    author            TEXT NOT NULL DEFAULT '',
    format            TEXT NOT NULL DEFAULT 'physical'
                      CHECK (format IN ('physical', 'ebook', 'audiobook', 'article_pdf', 'manga_webnovel')),
    status            TEXT NOT NULL DEFAULT 'want_to_read'
                      CHECK (status IN ('want_to_read', 'reading', 'paused', 'completed', 'abandoned')),
    progress_unit     TEXT NOT NULL DEFAULT 'pages'
                      CHECK (progress_unit IN ('pages', 'percent', 'minutes', 'chapters')),
    total_units       INTEGER NOT NULL DEFAULT 0 CHECK (total_units >= 0),
    current_units     INTEGER NOT NULL DEFAULT 0 CHECK (current_units >= 0),
    rating_half_stars INTEGER CHECK (rating_half_stars BETWEEN 1 AND 10 OR rating_half_stars IS NULL),
    genre             TEXT NOT NULL DEFAULT '',
    open_library_key  TEXT,
    cover_cache_key   TEXT REFERENCES asset_cache(cache_key) ON DELETE SET NULL,
    reread_count      INTEGER NOT NULL DEFAULT 0 CHECK (reread_count >= 0),
    started_at        TEXT,
    finished_at       TEXT,
    extra_json        TEXT NOT NULL DEFAULT '{}',
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT
);
CREATE INDEX IF NOT EXISTS idx_books_status
    ON books(status, updated_at DESC) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS bookshelves (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    extra_json  TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE IF NOT EXISTS bookshelf_items (
    bookshelf_id TEXT NOT NULL REFERENCES bookshelves(id) ON DELETE CASCADE,
    book_id      TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    added_at     TEXT NOT NULL,
    PRIMARY KEY (bookshelf_id, book_id)
);

CREATE TABLE IF NOT EXISTS book_reading_logs (
    id            TEXT PRIMARY KEY,
    book_id       TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    delta_units   INTEGER NOT NULL,
    current_units INTEGER NOT NULL CHECK (current_units >= 0),
    emotion       TEXT,
    comment       TEXT NOT NULL DEFAULT '',
    logged_at     TEXT NOT NULL,
    extra_json    TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_book_reading_logs_book
    ON book_reading_logs(book_id, logged_at DESC);

CREATE TABLE IF NOT EXISTS book_quotes (
    id         TEXT PRIMARY KEY,
    book_id    TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    page_ref   TEXT NOT NULL DEFAULT '',
    quote_text TEXT NOT NULL,
    comment    TEXT NOT NULL DEFAULT '',
    extra_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS reading_goals (
    id           TEXT PRIMARY KEY,
    period_type  TEXT NOT NULL CHECK (period_type IN ('annual', 'monthly')),
    period_key   TEXT NOT NULL,
    target_books INTEGER NOT NULL DEFAULT 0 CHECK (target_books >= 0),
    target_pages INTEGER NOT NULL DEFAULT 0 CHECK (target_pages >= 0),
    extra_json   TEXT NOT NULL DEFAULT '{}',
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    deleted_at   TEXT,
    UNIQUE (period_type, period_key)
);

-- Sessões de foco/estudo/leitura (após study_subjects, study_topics, tasks e books)
CREATE TABLE IF NOT EXISTS study_sessions (
    id               TEXT PRIMARY KEY,
    subject_id       TEXT REFERENCES study_subjects(id) ON DELETE SET NULL,
    topic_id         TEXT REFERENCES study_topics(id) ON DELETE SET NULL,
    task_id          TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    book_id          TEXT REFERENCES books(id) ON DELETE SET NULL,
    modality         TEXT NOT NULL DEFAULT 'focus'
                     CHECK (modality IN ('focus', 'theory', 'questions', 'review', 'reading')),
    duration_minutes INTEGER NOT NULL CHECK (duration_minutes > 0),
    questions_total  INTEGER NOT NULL DEFAULT 0 CHECK (questions_total >= 0),
    questions_right  INTEGER NOT NULL DEFAULT 0 CHECK (questions_right >= 0),
    session_date     TEXT NOT NULL,
    started_at       TEXT NOT NULL,
    ended_at         TEXT NOT NULL,
    extra_json       TEXT NOT NULL DEFAULT '{}',
    created_at       TEXT NOT NULL,
    deleted_at       TEXT
);
CREATE INDEX IF NOT EXISTS idx_study_sessions_date
    ON study_sessions(session_date DESC, id DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_study_sessions_subject
    ON study_sessions(subject_id, session_date DESC) WHERE deleted_at IS NULL;

-- ============================================================================
-- CAMADA 1: ANOTAÇÕES INTEGRADAS + FTS5
-- ============================================================================
CREATE TABLE IF NOT EXISTS notes (
    id         TEXT PRIMARY KEY,
    slug       TEXT NOT NULL UNIQUE,
    title      TEXT NOT NULL,
    kind       TEXT NOT NULL DEFAULT 'free'
               CHECK (kind IN ('free', 'daily', 'subject', 'book', 'project')),
    summary    TEXT NOT NULL DEFAULT '',
    body_md    TEXT NOT NULL DEFAULT '',
    note_date  TEXT,
    pinned     INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    extra_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_notes_active
    ON notes(kind, updated_at DESC, id DESC) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_notes_daily_unique
    ON notes(note_date) WHERE kind = 'daily' AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS note_revisions (
    id         TEXT PRIMARY KEY,
    note_id    TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    body_md    TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_note_revisions_note
    ON note_revisions(note_id, created_at DESC);

CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    summary,
    body_md,
    content='notes',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, summary, body_md)
    VALUES (new.rowid, new.title, new.summary, new.body_md);
END;

CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, summary, body_md)
    VALUES ('delete', old.rowid, old.title, old.summary, old.body_md);
END;

CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, summary, body_md)
    VALUES ('delete', old.rowid, old.title, old.summary, old.body_md);
    INSERT INTO notes_fts(rowid, title, summary, body_md)
    VALUES (new.rowid, new.title, new.summary, new.body_md);
END;

-- ============================================================================
-- CAMADA 2: ECONOMIA HMAC, GAMIFICAÇÃO, JARDIM, PET, CONQUISTAS & MCP
-- ============================================================================
CREATE TABLE IF NOT EXISTS daily_checkins (
    profile_id   TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    checkin_date TEXT NOT NULL,
    streak_count INTEGER NOT NULL CHECK (streak_count >= 1),
    used_freeze  INTEGER NOT NULL DEFAULT 0 CHECK (used_freeze IN (0, 1)),
    created_at   TEXT NOT NULL,
    PRIMARY KEY (profile_id, checkin_date)
);

CREATE TABLE IF NOT EXISTS wallet_ledger (
    id              TEXT PRIMARY KEY,
    profile_id      TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    seq             INTEGER NOT NULL CHECK (seq >= 1),
    prev_hash       TEXT NOT NULL,
    entry_type      TEXT NOT NULL CHECK (
                        entry_type IN (
                            'daily_checkin',
                            'pomodoro_focus',
                            'routine_goal',
                            'achievement_reward',
                            'shop_purchase',
                            'admin_grant',
                            'streak_freeze_purchase'
                        )
                    ),
    amount          INTEGER NOT NULL CHECK (amount != 0),
    reference_id    TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    hmac_sig        TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    UNIQUE (profile_id, seq)
);
CREATE INDEX IF NOT EXISTS idx_wallet_ledger_profile_seq
    ON wallet_ledger(profile_id, seq ASC);

CREATE TABLE IF NOT EXISTS inventory_items (
    id          TEXT PRIMARY KEY,
    profile_id  TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    item_kind   TEXT NOT NULL CHECK (item_kind IN ('theme', 'frame', 'pet', 'pet_accessory', 'plant_skin', 'garden_ground', 'dsp_track', 'streak_freeze', 'pet_treat')),
    item_key    TEXT NOT NULL,
    quantity    INTEGER NOT NULL DEFAULT 1 CHECK (quantity >= 0),
    acquired_at TEXT NOT NULL,
    UNIQUE (profile_id, item_kind, item_key)
);

CREATE TABLE IF NOT EXISTS cosmetic_loadout (
    profile_id TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    slot       TEXT NOT NULL CHECK (slot IN ('theme_preset', 'board_frame', 'active_pet', 'pet_accessory', 'garden_ground', 'dsp_track')),
    item_key   TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (profile_id, slot)
);

CREATE TABLE IF NOT EXISTS garden_plantings (
    id             TEXT PRIMARY KEY,
    profile_id     TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    session_id     TEXT REFERENCES study_sessions(id) ON DELETE SET NULL,
    plant_skin_key TEXT NOT NULL,
    planted_date   TEXT NOT NULL,
    grid_x         INTEGER NOT NULL CHECK (grid_x >= 0),
    grid_y         INTEGER NOT NULL CHECK (grid_y >= 0),
    watered_count  INTEGER NOT NULL DEFAULT 0 CHECK (watered_count >= 0),
    extra_json     TEXT NOT NULL DEFAULT '{}',
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_garden_plantings_date
    ON garden_plantings(profile_id, planted_date DESC);

CREATE TABLE IF NOT EXISTS pet_state (
    profile_id         TEXT PRIMARY KEY REFERENCES local_profiles(id) ON DELETE CASCADE,
    species_key        TEXT NOT NULL DEFAULT 'calico_cat',
    pet_name           TEXT NOT NULL DEFAULT 'Mochi',
    mood_state         TEXT NOT NULL DEFAULT 'happy' CHECK (mood_state IN ('happy', 'focused', 'sleepy')),
    affection_points   INTEGER NOT NULL DEFAULT 0 CHECK (affection_points >= 0),
    last_interacted_at TEXT,
    updated_at         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS achievement_progress (
    profile_id      TEXT NOT NULL REFERENCES local_profiles(id) ON DELETE CASCADE,
    achievement_key TEXT NOT NULL,
    current_value   INTEGER NOT NULL DEFAULT 0 CHECK (current_value >= 0),
    target_value    INTEGER NOT NULL CHECK (target_value > 0),
    unlocked_at     TEXT,
    pinned_slot     INTEGER CHECK (pinned_slot IN (1, 2, 3) OR pinned_slot IS NULL),
    updated_at      TEXT NOT NULL,
    PRIMARY KEY (profile_id, achievement_key)
);

CREATE TABLE IF NOT EXISTS user_preferences (
    key        TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mcp_audit_log (
    id            TEXT PRIMARY KEY,
    tool_name     TEXT NOT NULL,
    mode          TEXT NOT NULL CHECK (mode IN ('read_only', 'read_write', 'e2e')),
    args_json     TEXT NOT NULL,
    outcome       TEXT NOT NULL CHECK (outcome IN ('ok', 'denied', 'error')),
    error_message TEXT,
    executed_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_mcp_audit_time
    ON mcp_audit_log(executed_at DESC);
