FROM postgres:14-bullseye

ENV POSTGRES_DB=postgres
ENV POSTGRES_PASSWORD=password

COPY init.sql /docker-entrypoint-initdb.d/
