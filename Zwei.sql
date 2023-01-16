-- Database init script for Zwei
-- Author: Riven Skaye
-- Project: Zwei

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS 'prefixes'(
    'server' INTEGER PRIMARY KEY NOT NULL,
    'prefix' VARCHAR(5) NOT NULL DEFAULT ';'
);

CREATE TABLE IF NOT EXISTS 'servertags'(
    'tagid' INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    'serverid' INTEGER NOT NULL,
    'tagname' LONGTEXT NOT NULL,
    UNIQUE('serverid', 'tagname') ON CONFLICT FAIL
);

CREATE TABLE IF NOT EXISTS 'tagsubs'(
    'tagid' INTEGER NOT NULL,
    'userid' INTEGER NOT NULL,
    UNIQUE('tagid', 'userid') ON CONFLICT FAIL,
    FOREIGN KEY ('tagid') REFERENCES 'servertags'('tagid') ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS 'warnings'(
    'warnid' INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    'serverid' INTEGER NOT NULL,
    'userid' INTEGER NOT NULL,
    'message' LONGTEXT NOT NULL DEFAULT ''
);
