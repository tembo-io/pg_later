format:
	cargo fmt --all
	cargo clippy

# run in pgrx
run:
	PG_LATER_DATABASE=pg_later cargo pgrx run
