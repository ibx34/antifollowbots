-- Add migration script here

CREATE TABLE IF NOT EXISTS "good_users" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "github" BIGINT NOT NULL UNIQUE,
    "personal_token" VARCHAR,
    "reports" BIGINT DEFAULT 0,
    "created" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS "reported_users" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "github" BIGINT NOT NULL UNIQUE,
    "reported_by" BIGINT NOT NULL REFERENCES "good_users"("id"),
    "reported" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    "verdict" VARCHAR
);

CREATE TABLE IF NOT EXISTS "good_user_pa_token_verification" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "time" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    "related_user" BIGINT NOT NULL REFERENCES "good_users"("id"),
    "personal_token" VARCHAR NOT NULL
)