FROM postgres:16-bookworm as builder

RUN apt-get update \
    && apt-get install -y \
	ca-certificates \
	clang \
	curl \
	gcc \
	git \
	libssl-dev \
	make \
	pkg-config \
	postgresql-server-dev-16

COPY . .
# install pgmq
RUN make install-pgmq PGRX_PG_CONFIG=$(which pg_config)

# Install Rust dependencies
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN $HOME/.cargo/bin/rustup default stable

# install pgrx
ARG PGRX_VER=0.11.4
RUN $HOME/.cargo/bin/cargo install cargo-pgrx --version=$PGRX_VER --locked
RUN $HOME/.cargo/bin/cargo pgrx init --pg16 $(which pg_config)

# install pglater
RUN $HOME/.cargo/bin/cargo pgrx install --pg-config=$(which pg_config)

FROM postgres:16-bookworm

COPY --from=builder /usr/share/postgresql/16/extension /usr/share/postgresql/16/extension
COPY --from=builder /usr/lib/postgresql/16/lib /usr/lib/postgresql/16/lib

RUN apt-get update \
    && apt-get install -y ca-certificates

COPY images/pglater-pg/postgresql.conf /usr/share/postgresql/16/postgresql.conf.sample

USER postgres
CMD ["postgres"]
