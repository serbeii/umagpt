-- Game data (scraped, read-only after ingest)
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    rarity INTEGER,
    type TEXT,
    icon_url TEXT
);

CREATE TABLE IF NOT EXISTS support_cards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    character_name TEXT NOT NULL,
    rarity TEXT,
    type TEXT,
    base_stats TEXT, -- JSON
    skill_ids TEXT -- JSON
);

CREATE TABLE IF NOT EXISTS trainees (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    rarity INTEGER NOT NULL DEFAULT 1,
    base_stats TEXT, -- JSON
    aptitudes TEXT -- JSON
);

CREATE TABLE IF NOT EXISTS races (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    distance INTEGER,
    surface TEXT,
    direction TEXT,
    track_condition TEXT,
    scenario TEXT
);

CREATE TABLE IF NOT EXISTS trainee_events (
    id TEXT PRIMARY KEY,
    trainee_id TEXT REFERENCES trainees(id),
    event_name TEXT,
    choices TEXT -- JSON
);

CREATE TABLE IF NOT EXISTS scenario_info (
    scenario TEXT,      -- 'ura_finale' | 'unity_cup' | 'trackblazer'
    section  TEXT,      -- 'overview' | 'mechanics' | 'tips' | 'race_schedule'
    content  TEXT,
    embedding BLOB
);

-- Plan registry (static seed)
CREATE TABLE IF NOT EXISTS plans (
    id       TEXT PRIMARY KEY,
    kind     TEXT,    -- 'team_trials' | 'champions_meet' | 'scenario' | 'free'
    label    TEXT,
    meta     TEXT     -- JSON: tier, cup number, sign, etc.
);

-- User data (persistent, read-write)
CREATE TABLE IF NOT EXISTS owned_trainees (
    id              TEXT PRIMARY KEY,   -- matches trainees.id from game data
    nickname        TEXT,               -- optional user label
    star_rank       INTEGER NOT NULL DEFAULT 1,   -- 1–5 (white star count)
    potential_spd   INTEGER NOT NULL DEFAULT 0,   -- 0–10 per stat
    potential_sta   INTEGER NOT NULL DEFAULT 0,
    potential_pow   INTEGER NOT NULL DEFAULT 0,
    potential_gut   INTEGER NOT NULL DEFAULT 0,
    potential_wit   INTEGER NOT NULL DEFAULT 0,
    inherited_skills TEXT,              -- JSON: [skill_id, ...]
    parent_slot_1   TEXT,               -- references owned_trainees.id
    parent_slot_2   TEXT,
    notes           TEXT,
    created_at      TEXT DEFAULT (datetime('now')),
    updated_at      TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS owned_cards (
    id              TEXT PRIMARY KEY,   -- matches support_cards.id from game data
    card_level      INTEGER NOT NULL DEFAULT 1,   -- 1–50 (or 1–45 etc.)
    limit_break     INTEGER NOT NULL DEFAULT 0,   -- 0–4 (MLB = 4)
    bond_level      INTEGER NOT NULL DEFAULT 0,   -- 0–100
    notes           TEXT,
    created_at      TEXT DEFAULT (datetime('now')),
    updated_at      TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    title       TEXT,                -- auto-generated from first user message
    plan_id     TEXT,                -- plan active at time of conversation
    scenario    TEXT,
    trainee_id  TEXT,
    created_at  TEXT DEFAULT (datetime('now')),
    updated_at  TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT CHECK(role IN ('user','assistant','system')),
    content         TEXT,
    display_tags    TEXT,   -- JSON: parsed [DISPLAY:] tags for re-rendering
    created_at      TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS trial_teams (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    name        TEXT NOT NULL,          -- user's label, e.g. "Speed Team A"
    tier        TEXT NOT NULL,          -- 'beginner'|'intermediate'|'expert'|'master'
    notes       TEXT,
    created_at  TEXT DEFAULT (datetime('now')),
    updated_at  TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS trial_team_slots (
    id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    team_id     TEXT REFERENCES trial_teams(id) ON DELETE CASCADE,
    slot_index  INTEGER NOT NULL,       -- 0, 1, 2 (3 trainees per team)
    trainee_id  TEXT,                   -- references owned_trainees.id (nullable = empty slot)
    deck_json   TEXT,                   -- JSON: [card_id x 6] for this trainee's deck
    target_race TEXT,                   -- references races.id
    notes       TEXT
);

-- App state
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT
);
