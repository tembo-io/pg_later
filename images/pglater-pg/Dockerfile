FROM quay.io/tembo/coredb-pg-slim:latest as base
USER root
RUN rm /usr/share/postgresql/15/postgresql.conf.sample
COPY ./postgresql.conf /usr/share/postgresql/15/postgresql.conf.sample
RUN trunk install pgmq --version 0.11.2
RUN trunk install pg_later --version 0.0.9
ENV PGLATER_SOCKET_URL="postgresql:///postges?host=/var/run/postgresql/&user=postgres&dbname=postgres&port=5432"

USER postgres
CMD ["postgres"]
