# --- Stage 1: Build ---
FROM rust:trixie AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .

# Use a build argument to decide which binary to compile
ARG BIN_NAME
ARG PKG_NAME

# This tries to build the package; if it's a bin, it still works
RUN cargo build --release -p ${PKG_NAME} --bin ${BIN_NAME}
# --- Stage 2: Runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Re-declare the argument for the runtime stage
ARG BIN_NAME
WORKDIR /app
COPY --from=builder /app/target/release/${BIN_NAME} /usr/local/bin/app

# Start the selected app
CMD ["app"]