-- This file should undo anything in `up.sql`
ALTER TABLE subscription_tokens ALTER COLUMN generated_at DROP NOT NULL;
