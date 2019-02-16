CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE allmima (
    id          UUID            PRIMARY KEY,
    title       varchar(128)    NOT NULL DEFAULT '',
    username    varchar(128)    NOT NULL DEFAULT '',
    password    bytea,
    notes       bytea,
    favorite    boolean         NOT NULL DEFAULT 'f',
    created     timestamp       NOT NULL,
    deleted     timestamp       NOT NULL DEFAULT 'epoch',
    UNIQUE(title, username, deleted)
);

CREATE TABLE history (
    id          UUID            PRIMARY KEY,
    mima_id     UUID            NOT NULL
                                REFERENCES allmima(id) ON DELETE CASCADE,
    title       varchar(128)    NOT NULL DEFAULT '',
    username    varchar(128)    NOT NULL DEFAULT '',
    password    bytea,
    notes       bytea
);
