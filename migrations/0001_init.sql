PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS artists (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS albums (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
  id TEXT NOT NULL PRIMARY KEY,
  filepath TEXT NOT NULL,
  scan_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
  id TEXT NOT NULL PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  password TEXT NOT NULL,
  salt TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS folders (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  filepath TEXT NOT NULL
);

