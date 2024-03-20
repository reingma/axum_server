-- Your SQL goes here
    UPDATE subscription_tokens
        SET generated_at = timezone('utc', now())
        WHERE generated_at IS NULL;
    ALTER TABLE subscription_tokens ALTER COLUMN generated_at SET NOT NULL;
