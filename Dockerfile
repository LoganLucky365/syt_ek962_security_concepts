FROM rust:1.85-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/hash_password.rs

RUN cargo build --release || true

RUN rm -rf src

COPY src ./src

RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/syt_ek962_security_concepts /app/auth-service
COPY --from=builder /app/target/release/hash_password /app/hash_password

COPY initial_admin.json* /app/

RUN mkdir -p /app/data && chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_URL=sqlite:/app/data/auth.db?mode=rwc
ENV JWT_SECRET=""
ENV JWT_EXPIRATION_SECS=3600
ENV JWT_ISSUER=auth-service
ENV INITIAL_ADMIN_CONFIG=/app/initial_admin.json
ENV RUST_LOG=info,sqlx=warn

# Google OAuth (optional)
ENV GOOGLE_CLIENT_ID=""
ENV GOOGLE_CLIENT_SECRET=""
ENV GOOGLE_REDIRECT_URI=""

CMD ["/app/auth-service"]
