-- Add up migration script here
CREATE TABLE IF NOT EXISTS
    animation_plugins (
        id TEXT NOT NULL PRIMARY KEY,
        path TEXT NOT NULL,
        manifest BLOB NOT NULL
    );