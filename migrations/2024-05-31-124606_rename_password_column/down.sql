-- This file should undo anything in `up.sql`
ALTER TABLE users RENAME password_hash TO password;
