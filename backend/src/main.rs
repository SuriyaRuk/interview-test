use axum::{
    extract::Json as ExtractJson,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::env;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

mod api_tests;
mod file_demo;
mod models;
mod storage;

use models::*;
use storage::*;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = create_app();

    // Run it with hyper on localhost:8000
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("🚀 Semantic Search Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn create_app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/reviews", post(create_review))
        .route("/reviews/bulk", post(bulk_upload))
        .route("/search", post(search_reviews))
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        )
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "semantic-search-backend",
        "version": "0.1.0"
    }))
}

async fn create_review(
    ExtractJson(review_data): ExtractJson<ReviewData>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Validate the review data
    if let Err(validation_error) = review_data.validate() {
        let error_response = ErrorResponse::from(AppError::Validation(validation_error));
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Initialize data paths and storage
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "backend/data".to_string());
    let data_paths = DataPaths::new(&data_dir);

    // Ensure directories exist
    if let Err(e) = data_paths.ensure_directories() {
        let error_response = ErrorResponse::from(e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let jsonl_storage = JsonlStorage::new(&data_paths.reviews_jsonl);

    // Get current review count to determine vector index
    let vector_index = match jsonl_storage.count_reviews() {
        Ok(count) => count,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Convert to metadata with generated ID and timestamp
    let review_metadata = match review_data.to_metadata(vector_index) {
        Ok(metadata) => metadata,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Acquire file lock for concurrent safety
    let _lock = match FileLock::acquire(&data_paths.lock_file) {
        Ok(lock) => lock,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::SERVICE_UNAVAILABLE, Json(error_response)));
        }
    };

    // Store the review metadata in JSONL file
    if let Err(e) = jsonl_storage.append_review(&review_metadata) {
        let error_response = ErrorResponse::from(e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    // TODO: Generate embedding and store in vector index (Task 6 & 7)
    // For now, we'll just log that the vector would be stored
    tracing::info!(
        "Review stored successfully. Vector index {} would be stored in reviews.index",
        vector_index
    );

    // Return success response
    Ok(Json(json!({
        "success": true,
        "message": "Review created successfully",
        "review_id": review_metadata.id,
        "vector_index": vector_index,
        "timestamp": review_metadata.timestamp
    })))
}

async fn bulk_upload(
    ExtractJson(bulk_data): ExtractJson<Value>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize data paths and storage
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "backend/data".to_string());
    let data_paths = DataPaths::new(&data_dir);
    
    // Ensure directories exist
    if let Err(e) = data_paths.ensure_directories() {
        let error_response = ErrorResponse::from(e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let jsonl_storage = JsonlStorage::new(&data_paths.reviews_jsonl);

    // Acquire file lock for concurrent safety
    let _lock = match FileLock::acquire(&data_paths.lock_file) {
        Ok(lock) => lock,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::SERVICE_UNAVAILABLE, Json(error_response)));
        }
    };

    // Get current review count to determine starting vector index
    let starting_vector_index = match jsonl_storage.count_reviews() {
        Ok(count) => count,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Parse bulk data - support both array format and JSONL format
    let review_data_list: Vec<ReviewData> = match parse_bulk_data(&bulk_data) {
        Ok(reviews) => reviews,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    if review_data_list.is_empty() {
        let error_response = ErrorResponse::from(AppError::Validation(
            ValidationError::InvalidValue {
                field: "reviews".to_string(),
                reason: "No valid reviews found in bulk data".to_string(),
            }
        ));
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Process each review and collect results
    let mut successful_reviews = Vec::new();
    let mut failed_reviews = Vec::new();
    let mut current_vector_index = starting_vector_index;

    for (line_number, review_data) in review_data_list.iter().enumerate() {
        match process_single_review(review_data, current_vector_index) {
            Ok(metadata) => {
                successful_reviews.push(metadata);
                current_vector_index += 1;
            }
            Err(e) => {
                failed_reviews.push(BulkError {
                    line_number: line_number + 1,
                    error: e.to_string(),
                    data: Some(serde_json::to_value(review_data).unwrap_or(Value::Null)),
                });
            }
        }
    }

    // Store all successful reviews in batch
    if !successful_reviews.is_empty() {
        if let Err(e) = jsonl_storage.append_reviews(&successful_reviews) {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }

        // TODO: Generate embeddings and store in vector index (Task 6 & 7)
        tracing::info!(
            "Bulk upload: {} reviews stored successfully. Vector indices {}-{} would be stored in reviews.index",
            successful_reviews.len(),
            starting_vector_index,
            current_vector_index - 1
        );
    }

    // Create bulk upload result
    let bulk_result = BulkUploadResult {
        total_processed: review_data_list.len(),
        successful: successful_reviews.len(),
        failed: failed_reviews,
    };

    // Return success response with detailed results
    Ok(Json(json!({
        "success": true,
        "message": format!("Bulk upload completed: {} successful, {} failed", 
                          bulk_result.successful, bulk_result.failed.len()),
        "result": bulk_result,
        "starting_vector_index": starting_vector_index,
        "ending_vector_index": current_vector_index - 1
    })))
}

/// Parse bulk data from various formats (JSON array, JSONL, etc.)
fn parse_bulk_data(bulk_data: &Value) -> Result<Vec<ReviewData>, AppError> {
    match bulk_data {
        // Handle JSON array format: [{"title": "...", ...}, ...]
        Value::Array(reviews) => {
            let mut parsed_reviews = Vec::new();
            for review_value in reviews {
                match serde_json::from_value::<ReviewData>(review_value.clone()) {
                    Ok(review) => parsed_reviews.push(review),
                    Err(e) => {
                        return Err(AppError::Serialization(e));
                    }
                }
            }
            Ok(parsed_reviews)
        }
        // Handle single object wrapped in array
        Value::Object(_) => {
            match serde_json::from_value::<ReviewData>(bulk_data.clone()) {
                Ok(review) => Ok(vec![review]),
                Err(e) => Err(AppError::Serialization(e)),
            }
        }
        // Handle string format (JSONL)
        Value::String(jsonl_content) => {
            let mut parsed_reviews = Vec::new();
            for (line_num, line) in jsonl_content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ReviewData>(line) {
                    Ok(review) => parsed_reviews.push(review),
                    Err(e) => {
                        return Err(AppError::Validation(ValidationError::InvalidValue {
                            field: format!("line_{}", line_num + 1),
                            reason: format!("Invalid JSON: {}", e),
                        }));
                    }
                }
            }
            Ok(parsed_reviews)
        }
        _ => Err(AppError::Validation(ValidationError::InvalidValue {
            field: "bulk_data".to_string(),
            reason: "Expected JSON array, object, or JSONL string".to_string(),
        })),
    }
}

/// Process a single review and convert to metadata
fn process_single_review(review_data: &ReviewData, vector_index: usize) -> Result<ReviewMetadata, AppError> {
    // Validate the review data
    review_data.validate()?;
    
    // Convert to metadata with generated ID and timestamp
    review_data.to_metadata(vector_index)
}

async fn search_reviews(
    ExtractJson(search_request): ExtractJson<SearchRequest>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Validate the search request
    if let Err(validation_error) = search_request.validate() {
        let error_response = ErrorResponse::from(AppError::Validation(validation_error));
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Initialize data paths and storage
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "backend/data".to_string());
    let data_paths = DataPaths::new(&data_dir);
    
    // Ensure directories exist
    if let Err(e) = data_paths.ensure_directories() {
        let error_response = ErrorResponse::from(e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let jsonl_storage = JsonlStorage::new(&data_paths.reviews_jsonl);

    // Read all reviews for text-based search (TODO: Replace with vector search in Tasks 6 & 7)
    let all_reviews = match jsonl_storage.read_all_reviews() {
        Ok(reviews) => reviews,
        Err(e) => {
            let error_response = ErrorResponse::from(e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Perform text-based similarity search (placeholder for vector search)
    let search_results = perform_text_search(&search_request.query, &all_reviews, search_request.get_limit());

    // TODO: Generate query embedding and search vector index (Tasks 6 & 7)
    tracing::info!(
        "Search performed for query: '{}', found {} results",
        search_request.query,
        search_results.len()
    );

    // Return search results
    Ok(Json(json!({
        "success": true,
        "query": search_request.query,
        "results": search_results,
        "total_results": search_results.len(),
        "limit": search_request.get_limit(),
        "search_type": "text_similarity" // Will be "vector_similarity" after Tasks 6 & 7
    })))
}

/// Perform text-based similarity search (placeholder for vector search)
fn perform_text_search(query: &str, reviews: &[ReviewMetadata], limit: usize) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    
    if query_words.is_empty() {
        return Vec::new();
    }

    let mut scored_reviews: Vec<(ReviewMetadata, f32)> = reviews
        .iter()
        .map(|review| {
            let score = calculate_text_similarity(&query_lower, &query_words, review);
            (review.clone(), score)
        })
        .filter(|(_, score)| *score > 0.0) // Only include reviews with some similarity
        .collect();

    // Sort by similarity score in descending order
    scored_reviews.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top results up to limit
    scored_reviews
        .into_iter()
        .take(limit)
        .map(|(review, score)| SearchResult {
            review,
            similarity_score: score,
        })
        .collect()
}

/// Calculate text-based similarity score between query and review
fn calculate_text_similarity(query_lower: &str, query_words: &[&str], review: &ReviewMetadata) -> f32 {
    let title_lower = review.title.to_lowercase();
    let body_lower = review.body.to_lowercase();
    let combined_text = format!("{} {}", title_lower, body_lower);
    
    let mut score = 0.0;
    let total_words = query_words.len() as f32;
    
    // Exact phrase matching (highest weight)
    if combined_text.contains(query_lower) {
        score += 1.0;
    }
    
    // Individual word matching
    let mut word_matches = 0;
    for word in query_words {
        if combined_text.contains(word) {
            word_matches += 1;
            
            // Higher weight for title matches
            if title_lower.contains(word) {
                score += 0.8;
            } else {
                score += 0.5;
            }
        }
    }
    
    // Bonus for high word match ratio
    let word_match_ratio = word_matches as f32 / total_words;
    score += word_match_ratio * 0.5;
    
    // Bonus for rating (slight preference for higher-rated reviews)
    score += (review.rating as f32 - 3.0) * 0.1;
    
    // Normalize score to 0-1 range
    score.min(1.0).max(0.0)
}
