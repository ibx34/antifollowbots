-- Add migration script here

CREATE TABLE IF NOT EXISTS "sessions" (
    "id" BIGSERIAL NOT NULL PRIMARY KEY,
    "key" VARCHAR NOT NULL,
    "creator" BIGINT NOT NULL REFERENCES "good_users"("id"),
    "created_at" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
)