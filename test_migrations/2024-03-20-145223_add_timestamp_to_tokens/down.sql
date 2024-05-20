-- This file should undo anything in `up.sql`
ALTER TABLE subscription_tokens DROP COLUMN generated_at;
