-- This file should undo anything in `up.sql`
UPDATE users
	SET password_hash = '$argon2id$v=19$m=15000,t=2,p=1$v8TDJW/nkxwKV7VGGsffSw$gcKwJbSYBoEw7jQon4eqO1Yq6FgPMynhN2zMq8n4UCc'
	WHERE user_id = 'c9f4598c-e76c-46d4-811d-620478288ee7';
