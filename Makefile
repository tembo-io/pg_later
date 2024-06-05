DISTNAME = $(shell grep -m 1 '^name' Trunk.toml | sed -e 's/[^"]*"\([^"]*\)",\{0,1\}/\1/')
DISTVERSION  = $(shell grep -m 1 '^version' Cargo.toml | sed -e 's/[^"]*"\([^"]*\)",\{0,1\}/\1/')
PG_VERSION:=16
PGRX_PG_CONFIG =$(shell cargo pgrx info pg-config pg${PG_VERSION})
PGLATER_SOCKET_URL:='postgresql:///postgres?host=${HOME}/.pgrx&user=${USER}&dbname=postgres&port=288${PG_VERSION}'
DATABASE_URL:=postgres://${USER}:${USER}@localhost:288${PG_VERSION}/postgres

.PHONY: format run setup

format:
	cargo fmt --all
	cargo clippy

# run in pgrx locally
run:
	SQLX_OFFLINE=true PGLATER_SOCKET_URL=${PGLATER_SOCKET_URL} cargo pgrx run pg${PG_VERSION} postgres

META.json: META.json.in Trunk.toml
	@sed "s/@CARGO_VERSION@/$(DISTVERSION)/g" $< > $@

$(DISTNAME)-$(DISTVERSION).zip: META.json
	git archive --format zip --prefix $(DISTNAME)-$(DISTVERSION)/ --add-file $< -o $(DISTNAME)-$(DISTVERSION).zip HEAD

pgxn-zip: $(DISTNAME)-$(DISTVERSION).zip

clean:
	@rm -rf META.json $(DISTNAME)-$(DISTVERSION).zip

install-pgmq:
	git clone https://github.com/tembo-io/pgmq.git && \
	cd pgmq && \
	PG_CONFIG=${PGRX_PG_CONFIG} make clean && \
	PG_CONFIG=${PGRX_PG_CONFIG} make && \
	PG_CONFIG=${PGRX_PG_CONFIG} make install && \
	cd .. && rm -rf pgmq

setup.shared_preload_libraries:
	echo "shared_preload_libraries = 'pg_later'" >> ~/.pgrx/data-${PG_VERSION}/postgresql.conf

setup: install-pgmq setup.shared_preload_libraries 

test:
	SQLX_OFFLINE=true DATABASE_URL=${DATABASE_URL} cargo pgrx test
