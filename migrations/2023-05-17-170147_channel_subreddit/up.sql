-- Your SQL goes here
CREATE TABLE channel_subreddit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL,
    subreddit_id INTEGER NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES channel(id) ON DELETE CASCADE,
    FOREIGN KEY (subreddit_id) REFERENCES subreddit(id) ON DELETE CASCADE
);