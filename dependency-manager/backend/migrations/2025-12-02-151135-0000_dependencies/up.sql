-- Your SQL goes here
CREATE TABLE DEPENDENCIES (
    dependency_name TEXT NOT NULL PRIMARY KEY,
    dependency_source_path TEXT NOT NULL,
    dependency_version TEXT NOT NULL,
    author TEXT NOT NULL,
    date_added DATE NOT NULL DEFAULT NOW(),
    secret TEXT NOT NULL
)