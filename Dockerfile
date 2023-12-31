# Build stage
FROM node:20-alpine AS frontend
COPY frontend .
RUN yarn install
RUN yarn run vite build --outDir dist

# Separate Rust build
FROM rust:1.73 AS backend
COPY backend .
RUN cargo build --release --bin docker

# Production stage  
FROM rust:1.73

# Copy react static files
COPY --from=frontend dist static

# Copy rust binary and required files
COPY --from=backend target/release/docker app
COPY --from=backend .env .env
COPY --from=backend shuttle.db shuttle.db

ENTRYPOINT ["./app"]
