FROM quay.io/tembo/tembo-local:latest

ARG PG_LATER_VER=0.0.12
ARG PGMQ_VER=0.32.1

RUN trunk install pgmq --version $PGMQ_VER
RUN trunk install pg_later --version $PG_LATER_VER
