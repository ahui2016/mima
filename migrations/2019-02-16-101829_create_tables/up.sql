CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE allmima (
    id          varchar(32)     PRIMARY KEY
                                CHECK(char_length(id) = 32),
    title       varchar(128)    NOT NULL DEFAULT '',
    username    varchar(128)    NOT NULL DEFAULT '',
    password    bytea,
    p_nonce     bytea,
    notes       bytea,
    n_nonce     bytea,
    favorite    boolean         NOT NULL DEFAULT 'f',
    created     varchar(20)     NOT NULL CHECK(char_length(created) = 20),
    deleted     varchar(20)     NOT NULL DEFAULT '1970-01-01T00:00:00Z'
                                CHECK(char_length(deleted) = 20),
    UNIQUE(title, username, deleted)
);

CREATE TABLE history (
    id          varchar(32)     PRIMARY KEY
                                CHECK(char_length(id) = 32),
    mima_id     varchar(32)     NOT NULL
                                REFERENCES allmima(id) ON DELETE CASCADE,
    title       varchar(128)    NOT NULL DEFAULT '',
    username    varchar(128)    NOT NULL DEFAULT '',
    password    bytea,
    p_nonce     bytea,
    notes       bytea,
    n_nonce     bytea,
    favorite    boolean         NOT NULL DEFAULT 'f',
    deleted     varchar(20)     NOT NULL CHECK(char_length(deleted) = 20)
);
