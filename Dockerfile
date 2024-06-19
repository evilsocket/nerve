FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/nerve ./nerve

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/nerve /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/nerve"]