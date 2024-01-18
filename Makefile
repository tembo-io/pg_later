DISTVERSION  = $(shell grep -m 1 '^version' Cargo.toml | sed -e 's/[^"]*"\([^"]*\)",\{0,1\}/\1/')

format:
	cargo fmt --all
	cargo clippy

# run in pgrx locally
run:
	PGLATER_SOCKET_URL='postgresql:///pg_later?host=/Users/${USER}/.pgrx&user=${USER}&dbname=pg_later&port=28815' cargo pgrx run

META.json.bak: Cargo.toml META.json
	@sed -i.bak "s/@CARGO_VERSION@/$(DISTVERSION)/g" META.json

pgxn-zip: META.json.bak
	git archive --format zip --prefix=pg_later-$(DISTVERSION)/ -o pg_later-$(DISTVERSION).zip HEAD
	@mv META.json.bak META.json
