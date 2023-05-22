-- Your SQL goes here
CREATE TABLE channel (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id BIGINT NOT NULL UNIQUE,
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    title TEXT NOT NULL,
    username TEXT,
    invite_link TEXT
);