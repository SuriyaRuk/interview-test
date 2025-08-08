# Build stage
FROM rust:1.82 as builder

# Install wasm-pack for WebAssembly compilation
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY backend/ ./backend/
COPY frontend/ ./frontend/

# Set working directory to frontend for building
WORKDIR /app/frontend

# Build the WebAssembly package
RUN wasm-pack build --target web --out-dir pkg

# Runtime stage - serve with a simple HTTP server
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/frontend/pkg /usr/share/nginx/html/pkg
COPY --from=builder /app/frontend/style.css /usr/share/nginx/html/
COPY --from=builder /app/frontend/index.html /usr/share/nginx/html/

# Create enhanced nginx config with better error handling and asset serving
RUN echo 'server { \
    listen 3000; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    \
    # Enhanced logging for debugging \
    access_log /var/log/nginx/access.log; \
    error_log /var/log/nginx/error.log debug; \
    \
    # Global headers to prevent caching and enable debugging \
    add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
    add_header Pragma "no-cache" always; \
    add_header Expires "0" always; \
    add_header X-Content-Type-Options "nosniff" always; \
    add_header X-Frame-Options "DENY" always; \
    \
    # Custom error pages \
    error_page 404 /404.html; \
    error_page 500 502 503 504 /50x.html; \
    \
    # Main location block \
    location / { \
        try_files $uri $uri/ /index.html; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Last-Modified $date_gmt; \
        add_header ETag off; \
    } \
    \
    # WebAssembly package files \
    location /pkg/ { \
        add_header Access-Control-Allow-Origin "*" always; \
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always; \
        add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range" always; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Last-Modified $date_gmt; \
        \
        # Handle preflight requests \
        if ($request_method = OPTIONS) { \
            return 204; \
        } \
    } \
    \
    # WASM files with proper MIME type and headers \
    location ~* \.wasm$ { \
        add_header Content-Type "application/wasm" always; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Access-Control-Allow-Origin "*" always; \
        add_header Last-Modified $date_gmt; \
        expires off; \
    } \
    \
    # JavaScript files \
    location ~* \.js$ { \
        add_header Content-Type "application/javascript" always; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Last-Modified $date_gmt; \
        expires off; \
    } \
    \
    # CSS files \
    location ~* \.css$ { \
        add_header Content-Type "text/css" always; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Last-Modified $date_gmt; \
        expires off; \
    } \
    \
    # HTML files \
    location ~* \.html$ { \
        add_header Content-Type "text/html" always; \
        add_header Cache-Control "no-cache, no-store, must-revalidate" always; \
        add_header Last-Modified $date_gmt; \
        expires off; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 3000

# Start nginx
CMD ["nginx", "-g", "daemon off;"]