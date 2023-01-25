-- Database init script for Zwei
-- Author: Riven Skaye
-- Project: Zwei

-- PRAGMAs for sqlite3
-- use FKs
PRAGMA foreign_keys = ON;
-- Write Ahead Logging, for increased throughput from async code
PRAGMA journal_mode = WAL;


-- Prefixes the bot will respond to
CREATE TABLE IF NOT EXISTS 'prefixes'(
    'server' INTEGER PRIMARY KEY NOT NULL,
    'prefix' VARCHAR(5) NOT NULL DEFAULT ';'
);

-- Tags registered for a guild
CREATE TABLE IF NOT EXISTS 'servertags'(
    'tagid' INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    'serverid' INTEGER NOT NULL,
    'tagname' LONGTEXT NOT NULL,
    UNIQUE('serverid', 'tagname') ON CONFLICT FAIL
);

-- Users subscribed to certain tags
CREATE TABLE IF NOT EXISTS 'tagsubs'(
    'tagid' INTEGER NOT NULL,
    'userid' INTEGER NOT NULL,
    UNIQUE('tagid', 'userid') ON CONFLICT FAIL,
    FOREIGN KEY ('tagid') REFERENCES 'servertags'('tagid') ON DELETE CASCADE
);

-- Warnings emitted per user per guild
CREATE TABLE IF NOT EXISTS 'warnings'(
    'warnid' INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    'serverid' INTEGER NOT NULL,
    'userid' INTEGER NOT NULL,
    'message' LONGTEXT NOT NULL DEFAULT ''
);
