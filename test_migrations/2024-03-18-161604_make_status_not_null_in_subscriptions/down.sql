-- This file should undo anything in `up.sql`
ALTER TABLE subscriptions ALTER COLUMN status DROP NOT NULL;
