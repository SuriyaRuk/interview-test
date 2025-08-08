# Build stage
FROM rust:1.83 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml  ./
COPY backend/Cargo.toml ./backend/
COPY backend/src ./backend/src/
COPY frontend/Cargo.toml ./frontend/
COPY frontend/src ./frontend/src/

# Build the application
RUN cargo build --release --bin semantic-search-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Create data directory
RUN mkdir -p /app/data && chown appuser:appuser /app/data

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/semantic-search-backend /app/

# Change ownership
RUN chown appuser:appuser /app/semantic-search-backend

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8000

# Set environment variables
ENV DATA_DIR=/app/data

# Run the binary
CMD ["./semantic-search-backend"]
