# syntax=docker/dockerfile:1
# RepoHub in Docker: builds the web app (Rust → WebAssembly) and repohub-server, then runs the
# server, which serves the app on port 8080 and proxies the GitHub API with your token.
# Usage: see README.md ("Docker").

FROM rust:1-bookworm AS build
WORKDIR /src
# The toolchain version is pinned by rust-toolchain.toml; rustup installs it on first use.
COPY rust-toolchain.toml ./
RUN rustup target add wasm32-unknown-unknown \
 && cargo install wasm-bindgen-cli --version 0.2.129 --locked
COPY . .
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=s \
    CARGO_PROFILE_RELEASE_LTO=thin \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_DEBUG=0 \
    CARGO_PROFILE_RELEASE_STRIP=debuginfo
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/src/server/target \
    cargo build --release --lib --target wasm32-unknown-unknown \
 && wasm-bindgen --target web --out-dir /out/web --out-name app target/wasm32-unknown-unknown/release/repo_hub.wasm \
 && cp web/index.html /out/web/ \
 && cargo build --release --manifest-path server/Cargo.toml \
 && cp server/target/release/repohub-server /out/

FROM debian:bookworm-slim
# CA certificates for HTTPS to GitHub, taken from the build image (no package manager needed).
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /out /app
ENV REPOHUB_ADDR=0.0.0.0:8080 \
    WEB_DIR=/app/web
EXPOSE 8080
USER nobody
CMD ["/app/repohub-server"]
