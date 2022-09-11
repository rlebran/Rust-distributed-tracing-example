# Distributed tracing with Rust

Distributed tracing with Rust, Actix, Lapin with context propagation between multiple services.

More [on this article](http://google.com).

## Start the services:

First, start postgres, rabbit and zipkin with docker compose:
- `DOCKER_BUILDKIT=1 docker-compose -f docker-compose.yml up --build`

Then start both services:
- `DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/postgres cargo run --bin service1`
- `cargo run --bin service2`


## Test

To send a test request:
- `curl -d '{"content": "write an article about distributed tracing", "owner": "rlebran"}' -H "Content-Type: application/json" -X POST http://localhost:8081/v1/todo`
