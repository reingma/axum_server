-- Your SQL goes here
CREATE TYPE header_pair AS (
	name TEXT,
	value BYTEA
);

CREATE TYPE http_request AS (
	response_status_code SMALLINT ,
	response_headers header_pair[] ,
	reponse_body BYTEA ,
	http_version TEXT
);

CREATE TABLE idempotency (
	user_id uuid NOT NULL REFERENCES users(user_id),
	idempotency_key TEXT NOT NULL,
	request http_request NOT NULL, created_at timestamptz NOT NULL,
	PRIMARY KEY(user_id,idempotency_key)
);
