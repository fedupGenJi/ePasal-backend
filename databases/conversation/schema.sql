CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sender TEXT NOT NULL CHECK (sender IN ('user', 'admin', 'bot')),
    receiver TEXT NOT NULL CHECK (receiver IN ('user', 'admin', 'bot'))
);

CREATE TABLE IF NOT EXISTS user_bot_settings (
    user_id TEXT PRIMARY KEY,
    bot_enabled BOOLEAN NOT NULL DEFAULT TRUE
);