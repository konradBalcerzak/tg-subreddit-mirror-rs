-- Your SQL goes here
CREATE TABLE subreddit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    subreddit_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE,
    sorting TEXT NOT NULL DEFAULT "hot",
    post_limit INTEGER,
    respect_external_content_flag BOOLEAN NOT NULL DEFAULT FALSE,
    min_score INTEGER,
    allow_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    show_spoilers BOOLEAN NOT NULL DEFAULT FALSE,
    medias_only BOOLEAN NOT NULL DEFAULT FALSE
);