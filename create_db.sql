-- Creator:       MySQL Workbench 8.0.20/ExportSQLite Plugin 0.1.0
-- Author:        Riven
-- Caption:       Database creation script for Zwei
-- Project:       Zwei
-- Changed:       2021-04-20 20:26
-- Created:       2021-04-20 19:35

-- Schema: zweiDB
ATTACH "zweiDB.sdb" AS "zweiDB";
BEGIN;
CREATE TABLE "zweiDB"."Warnings"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "serverID" REAL,
  "userID" REAL,
  "Reason" LONGTEXT,
  CONSTRAINT "warnID_UNIQUE"
    UNIQUE("id")
);
CREATE TABLE "zweiDB"."Reminders"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "message" LONGTEXT,
  "serverID" REAL,
  "channelID" REAL,
  "triggertime" VARCHAR(45),
  CONSTRAINT "id_UNIQUE"
    UNIQUE("id")
);
CREATE TABLE "zweiDB"."RSS_Feeds"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "url" LONGTEXT,
  CONSTRAINT "id_UNIQUE"
    UNIQUE("id")
);
CREATE TABLE "zweiDB"."Webhook-feed"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "webhook_url" LONGTEXT,
  "RSS_Feeds_id" INTEGER NOT NULL,
  CONSTRAINT "updateID_UNIQUE"
    UNIQUE("id"),
  CONSTRAINT "fk_Webhook-feed_RSS_Feeds"
    FOREIGN KEY("RSS_Feeds_id")
    REFERENCES "RSS_Feeds"("id")
    ON DELETE CASCADE
    ON UPDATE CASCADE
);
CREATE INDEX "zweiDB"."Webhook-feed.fk_Webhook-feed_RSS_Feeds_idx" ON "Webhook-feed" ("RSS_Feeds_id");
CREATE TABLE "zweiDB"."Repeaters"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "message" LONGTEXT,
  "starttime" VARCHAR(45),
  "interval" INTEGER,
  "serverID" REAL,
  "channelID" REAL,
  "Repeaterscol" VARCHAR(45),
  CONSTRAINT "id_UNIQUE"
    UNIQUE("id")
);
CREATE TABLE "zweiDB"."Songs"(
  "id" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  "anime" VARCHAR(45),
  "type" VARCHAR(45),
  "artist" VARCHAR(45),
  "title" VARCHAR(45),
  "link" VARCHAR(45),
  CONSTRAINT "id_UNIQUE"
    UNIQUE("id")
);
COMMIT;
