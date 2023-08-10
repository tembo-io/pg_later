format:
	cargo fmt --all
	cargo clippy

# run in pgrx locally
run:
	PGLATER_SOCKET_URL='postgresql:///pg_later?host=/Users/${USER}/.pgrx&user=${USER}&dbname=pg_later&port=28815' cargo pgrx run