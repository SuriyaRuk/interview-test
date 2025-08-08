# Semantic Search Platform

A file-based semantic search platform for product reviews using vector embeddings.

## Architecture

- **Frontend**: Leptos (Rust WebAssembly)
- **Backend**: Axum (Rust)
- **Embeddings**: fastembed-rs
- **Vector Index**: SPFresh
- **Storage**: File-based (JSONL + binary index)

## Project Structure

```
├── backend/           # Axum backend server
├── frontend/          # Leptos frontend application
├── data/              # Data storage directory
│   ├── reviews.jsonl  # Review metadata (created at runtime)
│   └── reviews.index  # Vector index (created at runtime)
└── Cargo.toml         # Workspace configuration
```

## Development

This project uses a Cargo workspace. To build:

```bash
# Build entire workspace
cargo build

# Build backend only
cargo build -p semantic-search-backend

# Build frontend only
cargo build -p semantic-search-frontend
```

## Features

- Add product reviews through web interface
- Bulk upload reviews via file upload
- Semantic search using natural language queries
- Vector-based similarity matching
- File-based storage (no database required)
- Concurrent operation support
- Docker containerization

## API Specification

The backend provides a RESTful API for managing product reviews and performing semantic search operations.

### Base URL
```
http://localhost:8000
```

### Endpoints

#### Health Check
**GET** `/health`

Check the health status of the API server.

**Response:**
```json
{
  "status": "healthy",
  "service": "semantic-search-backend",
  "version": "0.1.0"
}
```

---

#### Create Review
**POST** `/reviews`

Add a single product review to the system.

**Request Body:**
```json
{
  "title": "Great product!",
  "body": "This product exceeded my expectations. Great quality and fast delivery.",
  "product_id": "prod_123",
  "rating": 5
}
```

**Validation Rules:**
- `title`: Required, 3-200 characters
- `body`: Required, 10-2000 characters  
- `product_id`: Required, max 100 characters
- `rating`: Required, integer 1-5

**Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Review created successfully",
  "review_id": "550e8400-e29b-41d4-a716-446655440000",
  "vector_index": 0,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Error Response (400 Bad Request):**
```json
{
  "error": "validation_error",
  "message": "Missing required field: title",
  "details": null,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

#### Bulk Upload Reviews
**POST** `/reviews/bulk`

Upload multiple reviews at once. Supports JSON array, single object, or JSONL string formats.

**Request Body (JSON Array):**
```json
[
  {
    "title": "Great product 1",
    "body": "This is the first review in bulk upload.",
    "product_id": "prod_001",
    "rating": 5
  },
  {
    "title": "Good product 2",
    "body": "This is the second review in bulk upload.",
    "product_id": "prod_002",
    "rating": 4
  }
]
```

**Request Body (JSONL String):**
```json
"{\"title\": \"JSONL Review 1\", \"body\": \"First review in JSONL format.\", \"product_id\": \"jsonl_001\", \"rating\": 5}\n{\"title\": \"JSONL Review 2\", \"body\": \"Second review in JSONL format.\", \"product_id\": \"jsonl_002\", \"rating\": 4}"
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Bulk upload completed: 2 successful, 0 failed",
  "result": {
    "total_processed": 2,
    "successful": 2,
    "failed": []
  },
  "starting_vector_index": 0,
  "ending_vector_index": 1
}
```

**Partial Success Response (200 OK):**
```json
{
  "success": true,
  "message": "Bulk upload completed: 2 successful, 1 failed",
  "result": {
    "total_processed": 3,
    "successful": 2,
    "failed": [
      {
        "line_number": 2,
        "error": "Missing required field: title",
        "data": {
          "title": "",
          "body": "Invalid review",
          "product_id": "prod_002",
          "rating": 4
        }
      }
    ]
  },
  "starting_vector_index": 0,
  "ending_vector_index": 1
}
```

---

#### Search Reviews
**POST** `/search`

Search for reviews using natural language queries with text-based similarity matching.

**Request Body:**
```json
{
  "query": "camera quality",
  "limit": 10
}
```

**Parameters:**
- `query`: Required, search query string (max 500 characters)
- `limit`: Optional, number of results to return (1-100, default: 10)

**Success Response (200 OK):**
```json
{
  "success": true,
  "query": "camera quality",
  "results": [
    {
      "review": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Amazing smartphone",
        "body": "This phone has excellent camera quality and fast performance. Great value for money.",
        "product_id": "phone_001",
        "rating": 5,
        "timestamp": "2024-01-15T10:30:00Z",
        "vector_index": 0
      },
      "similarity_score": 0.95
    }
  ],
  "total_results": 1,
  "limit": 10,
  "search_type": "text_similarity"
}
```

**No Results Response (200 OK):**
```json
{
  "success": true,
  "query": "nonexistent product",
  "results": [],
  "total_results": 0,
  "limit": 10,
  "search_type": "text_similarity"
}
```

---

### Error Responses

All endpoints return structured error responses with appropriate HTTP status codes:

**400 Bad Request - Validation Error:**
```json
{
  "error": "validation_error",
  "message": "Missing required field: title",
  "details": null,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**500 Internal Server Error - System Error:**
```json
{
  "error": "file_operation_error",
  "message": "File operation failed",
  "details": {
    "io_error": "Permission denied"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**503 Service Unavailable - Concurrency Error:**
```json
{
  "error": "concurrency_error",
  "message": "Failed to acquire file lock",
  "details": null,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

### Search Algorithm

The current implementation uses **text-based similarity matching** with the following features:

- **Exact phrase matching**: Highest priority for exact query matches
- **Individual word matching**: Matches individual words with title preference
- **Word coverage bonus**: Higher scores for queries with more word matches
- **Rating preference**: Slight preference for higher-rated reviews
- **Score normalization**: All scores normalized to 0-1 range
- **Ranking**: Results sorted by similarity score in descending order

*Note: This will be upgraded to vector-based semantic search using fastembed-rs and SPFresh in future releases.*

---

### Data Storage

The system uses a file-based storage approach:

- **reviews.jsonl**: Review metadata in JSONL format (one review per line)
- **reviews.index**: Vector index file for semantic search (future implementation)
- **Concurrent safety**: File locking prevents data corruption during concurrent operations
- **Zero-based indexing**: Vector index correlates directly with JSONL line numbers

## Getting Started

### Prerequisites
- Rust 1.75 or later
- Docker (optional, for containerized deployment)

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd semantic-search-platform
   ```

2. **Build the workspace:**
   ```bash
   cargo build
   ```

3. **Run the backend server:**
   ```bash
   cargo run -p semantic-search-backend
   ```
   The API will be available at `http://localhost:8000`

4. **Run the frontend (in a separate terminal):**

   **⚠️ Important:** The frontend is a Leptos web application that compiles to WebAssembly and must be served through a web server, not run as a standalone binary.
   
   **Option A: Using Docker (recommended):**
   ```bash
   # Build and run the frontend container
   docker build -t semantic-search-frontend ./frontend
   docker run -p 3000:3000 semantic-search-frontend
   ```
   
   **Option B: Manual WebAssembly build and serve:**
   ```bash
   # Install wasm-pack if not already installed
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   
   # Build the WebAssembly package
   cd frontend
   wasm-pack build --target web --out-dir pkg
   
   # Serve with a simple HTTP server (choose one):
   
   # Using Python 3
   python3 -m http.server 3000
   
   # Using Node.js (if you have npx)
   npx serve -p 3000 .
   
   # Using Rust basic-http-server (install with: cargo install basic-http-server)
   basic-http-server -a 0.0.0.0:3000 .
   ```
   
   **Option C: Using trunk for development (install with: cargo install trunk):**
   ```bash
   cd frontend
   trunk serve --port 3000
   ```
   
   The web interface will be available at `http://localhost:3000`

### Docker Deployment

1. **Build and run with Docker Compose:**
   ```bash
   docker-compose up --build
   ```

2. **Access the application:**
   - Frontend: `http://localhost:3000`
   - Backend API: `http://localhost:8000`

### Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run backend tests only
cargo test -p semantic-search-backend

# Run frontend tests only
cargo test -p semantic-search-frontend
```

### Example Usage

**Add a single review:**
```bash
curl -X POST http://localhost:8000/reviews \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Great product!",
    "body": "This product exceeded my expectations. Great quality and fast delivery.",
    "product_id": "prod_123",
    "rating": 5
  }'
```

**Search for reviews:**
```bash
curl -X POST http://localhost:8000/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "camera quality",
    "limit": 10
  }'
```

**Check API health:**
```bash
curl http://localhost:8000/health
```