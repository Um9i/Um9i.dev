# syntax=docker/dockerfile:1

# ---- build ----------------------------------------------------------------
FROM rust:1-slim-trixie AS build
WORKDIR /app

# Build dependencies first so they're cached until Cargo.toml/Cargo.lock change.
COPY Cargo.toml Cargo.lock clippy.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

# askama templates and site content are compiled into the binary.
COPY src ./src
COPY templates ./templates
COPY content ./content
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# ---- runtime --------------------------------------------------------------
FROM gcr.io/distroless/cc-debian13:nonroot
WORKDIR /app
COPY --from=build /app/target/release/um9i /app/um9i
COPY static ./static

ENV PORT=8080 RUST_LOG=um9i=info,tower_http=info
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/app/um9i"]
