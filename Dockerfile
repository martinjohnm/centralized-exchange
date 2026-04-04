# --- Stage 1: Build ---
FROM rust:trixie AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .

# Use a build argument to decide which binary to compile
ARG APP_NAME
RUN if [ "$APP_NAME" = "fire_orders" ]; then \
        cargo build --release -p engine --bin fire_orders; \
    else \
        cargo build --release -p ${APP_NAME}; \
    fi

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Re-declare the argument for the runtime stage
ARG APP_NAME
WORKDIR /app
COPY --from=builder /app/target/release/${APP_NAME} /usr/local/bin/app

# Start the selected app
CMD ["app"]