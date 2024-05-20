-- This file should undo anything in `up.sql`
ALTER TABLE subscription_tokens ADD COLUMN subscription_token TEXT NOT NULL;
