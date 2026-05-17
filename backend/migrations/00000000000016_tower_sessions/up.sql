CREATE TABLE auth_sessions (
    id          TEXT PRIMARY KEY NOT NULL,
    data        BLOB NOT NULL,
    expiry_date BIGINT NOT NULL
);

CREATE INDEX auth_sessions_expiry_date_idx ON auth_sessions(expiry_date);
