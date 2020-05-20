CREATE TABLE users
(
    id                UUID PRIMARY KEY,
    username          VARCHAR   NOT NULL UNIQUE,
    email             VARCHAR   NOT NULL UNIQUE,
    PASSWORD          BYTEA     NOT NULL,
    signed_up_at      TIMESTAMP NOT NULL,
    last_signed_in_at TIMESTAMP NOT NULL
);

CREATE FUNCTION sign_up(id UUID, username VARCHAR, email VARCHAR, password BYTEA)
    RETURNS users
AS
$$
INSERT INTO users (id, username, email, password, signed_up_at, last_signed_in_at)
values (id, username, email, password, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
RETURNING
    *
$$
    LANGUAGE SQL
    STRICT;

CREATE FUNCTION sign_in(username_or_email VARCHAR, pass BYTEA)
    RETURNS users
AS
$$
UPDATE
    users
SET last_signed_in_at = CURRENT_TIMESTAMP
WHERE username_or_email IN (username, email)
  AND password = pass
RETURNING
    *
$$
    LANGUAGE SQL
    STRICT;

CREATE FUNCTION remove_account(user_id UUID, pass BYTEA)
    RETURNS BOOLEAN
AS
$$
DELETE
FROM users
WHERE id = user_id
  AND PASSWORD = pass
RETURNING
    TRUE
$$
    LANGUAGE SQL;
