# rust-actix-graphql-sqlx-postgresql

Using GraphQL with Rust and Apollo Router

## Walkthrough

This repo and accompanying information was presented
at [RustLang Los Angeles February 2021 - see video](https://youtu.be/hMIL12Mj7Pw)

This talk covered:

- Core concepts of GraphQL
- Core concepts of GraphQL federation vs schema stitching
- Rust
- SQLx + PostgreSQL
- async-graphql
- actix
- Apollo Router

## Version

See [Cargo.toml](Cargo.toml) version

## Platforms / Technologies

* [Rust](https://www.rust-lang.org/en-US/)
* [Cargo](https://doc.rust-lang.org/cargo/)
* [Actix](https://actix.rs/)
* [GraphQL](https://graphql.org/)
* [Apollo GraphQL](https://www.apollographql.com/)
* [Node.js](https://nodejs.org/en/)

## Servers

### Optional - Setup Local PostGreSQL database

>      $ ./scripts/docker/init-db.sh

### Optional - Setup SQLx cli

>      $ cargo install sqlx-cli

### Run user microservice

>      $ cd ./svc-user
>      $ sqlx database create
>      $ sqlx migrate run
>      $ cargo run

### Run skill microservice

>      $ cd ./svc-skill
>      $ sqlx migrate run
>      $ cargo run

### Prepare supergraph

`https://www.apollographql.com/docs/rover/getting-started`

> rover supergraph compose --config rover/supergraph.yaml > router/supergraph-schema-compose.graphql

### Run Router
`https://www.apollographql.com/docs/router/quickstart`

> ./router --supergraph router/supergraph-schema-compose.graphql -c router/router.yaml
