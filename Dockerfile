# Multi-stage build for constitution-server.
#
# Stage 1: build the wasm32 frontend bundle + the binary archive + the
#          native server binary, all from the same source tree.
# Stage 2: distroless runtime carrying the server binary, the binary
#          archive, and the static frontend (with the wasm pkg).
#
# Build:
#   docker build -t constitution-server .
# Run:
#   docker run --rm -p 8080:8080 constitution-server

# ---------------------------------------------------------------------------
# Stage 1: builder
# ---------------------------------------------------------------------------
FROM rust:1.82-bookworm AS builder

# wasm-bindgen-cli version must match the wasm-bindgen dep in Cargo.toml.
ARG WASM_BINDGEN_VERSION=0.2.121

WORKDIR /src
RUN rustup target add wasm32-unknown-unknown \
    && cargo install --locked --version "${WASM_BINDGEN_VERSION}" wasm-bindgen-cli

# Cache deps by copying manifests first.
COPY Cargo.toml Cargo.lock ./
COPY crates/constitution-archive/Cargo.toml crates/constitution-archive/Cargo.toml
COPY crates/constitution-wasm/Cargo.toml crates/constitution-wasm/Cargo.toml
COPY crates/constitution-cli/Cargo.toml crates/constitution-cli/Cargo.toml
COPY crates/constitution-server/Cargo.toml crates/constitution-server/Cargo.toml
RUN mkdir -p crates/constitution-archive/src crates/constitution-wasm/src \
    crates/constitution-cli/src crates/constitution-server/src \
    && echo "fn main() {}" > crates/constitution-cli/src/main.rs \
    && echo "fn main() {}" > crates/constitution-server/src/main.rs \
    && echo "" > crates/constitution-archive/src/lib.rs \
    && echo "" > crates/constitution-wasm/src/lib.rs \
    && echo "" > crates/constitution-server/src/lib.rs \
    && cargo fetch

# Now copy the real sources.
COPY crates ./crates
COPY data ./data
COPY config ./config
COPY frontend ./frontend
COPY scripts ./scripts

# Build the binary archive (uses scripts via cargo).
RUN cargo build --release --bin constitution-archive \
    && target/release/constitution-archive build \
        --corpus data/chunks/constitution_full_corpus.json \
        --timeline data/process_timeline.json \
        --output data/index/constitution_archive.bin

# Build the wasm bundle.
RUN cargo build --release --target wasm32-unknown-unknown -p constitution-wasm \
    && wasm-bindgen \
        --target web \
        --out-dir frontend/wasm/pkg \
        --out-name constitution_wasm \
        target/wasm32-unknown-unknown/release/constitution_wasm.wasm

# Build the server binary.
RUN cargo build --release -p constitution-server

# ---------------------------------------------------------------------------
# Stage 2: runtime
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /srv
COPY --from=builder /src/target/release/constitution-server /usr/local/bin/constitution-server
COPY --from=builder /src/data/index/constitution_archive.bin /srv/data/index/constitution_archive.bin
COPY --from=builder /src/frontend /srv/frontend

ENV ARCHIVE_PATH=/srv/data/index/constitution_archive.bin
ENV STATIC_DIR=/srv/frontend
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
USER nonroot

ENTRYPOINT ["/usr/local/bin/constitution-server"]
