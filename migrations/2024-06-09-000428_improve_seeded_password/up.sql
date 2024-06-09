-- Your SQL goes here
UPDATE users
	SET password_hash = '$argon2id$v=19$m=15000,t=2,p=1$fvhmK//LNM9SR3yOrcM9sw$c5v2kqRGsZ8hBfrOwsnfb/ttinhbu2MLgtEO10T5nzo'
	WHERE user_id = 'c9f4598c-e76c-46d4-811d-620478288ee7';
