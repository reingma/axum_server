-- Your SQL goes here
CREATE TABLE Users (
	user_id uuid PRIMARY KEY,
	username TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL
);
