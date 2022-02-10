-- Database init script for Zwei
-- Author: Riven Skaye
-- Project: Zwei

ATTACH "Zwei.sdb" as "ZweiDB";
BEGIN;
CREATE TABLE IF NOT EXISTS "ZweiDB"."prefixes"(
    "server" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "prefix" VARCHAR(5) NOT NULL DEFAULT ";"
);

CREATE TABLE IF NOT EXISTS "ZweiDB"."servertags"(
    "tagid" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    "serverid" INTEGER NOT NULL,
    "tagname" LONGTEXT NOT NULL,
    CONSTRAINT "tagunique"
        UNIQUE("serverid", "tagname")
);

CREATE TABLE IF NOT EXISTS "ZweiDB"."tagsubs"(
    "subid" INTEGER NOT NULL,
    "userid" INTEGER NOT NULL,
    CONSTRAINT "subunique"
        UNIQUE("tagid", "userid"),
    FOREIGN KEY ("guildid") REFERENCES "servertags" ("tagid")
);
-- Oh and I figured a better way to handle the sub/tag thing!
-- A table for tags per server, with an ID and a table for users subbed
-- to each tag id. I'll handle caching it later
