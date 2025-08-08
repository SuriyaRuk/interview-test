use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_app;
    use tempfile::TempDir;
    use std::env;

    #[tokio::test]
    async fn test_create_review_endpoint() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/create_review", temp_path));

        let app = create_app();

        // Create a valid review request
        let review_data = json!({
            "title": "Great product!",
            "body": "This product exceeded my expectations. Great quality and fast delivery.",
            "product_id": "prod_123",
            "rating": 5
        });

        let request = Request::builder()
            .method("POST")
            .uri("/reviews")
            .header("content-type", "application/json")
            .body(Body::from(review_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["message"], "Review created successfully");
        assert!(response_json["review_id"].is_string());
        assert_eq!(response_json["vector_index"], 0); // First review should have index 0
    }

    #[tokio::test]
    async fn test_create_review_validation_error() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/validation_error", temp_path));

        let app = create_app();

        // Create an invalid review request (missing title)
        let invalid_review_data = json!({
            "title": "",
            "body": "This product exceeded my expectations.",
            "product_id": "prod_123",
            "rating": 5
        });

        let request = Request::builder()
            .method("POST")
            .uri("/reviews")
            .header("content-type", "application/json")
            .body(Body::from(invalid_review_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["error"], "validation_error");
        assert!(response_json["message"].as_str().unwrap().contains("title"));
    }

    #[tokio::test]
    async fn test_create_review_invalid_rating() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/invalid_rating", temp_path));

        let app = create_app();

        // Create a review with invalid rating
        let invalid_review_data = json!({
            "title": "Great product!",
            "body": "This product exceeded my expectations.",
            "product_id": "prod_123",
            "rating": 6  // Invalid rating (should be 1-5)
        });

        let request = Request::builder()
            .method("POST")
            .uri("/reviews")
            .header("content-type", "application/json")
            .body(Body::from(invalid_review_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["error"], "validation_error");
        assert!(response_json["message"].as_str().unwrap().contains("rating"));
    }

    #[tokio::test]
    async fn test_bulk_upload_json_array() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/bulk_json_{}", temp_path, std::process::id()));

        let app = create_app();

        // Create bulk upload data as JSON array
        let bulk_data = json!([
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
            },
            {
                "title": "Average product 3",
                "body": "This is the third review in bulk upload.",
                "product_id": "prod_003",
                "rating": 3
            }
        ]);

        let request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(bulk_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["result"]["total_processed"], 3);
        assert_eq!(response_json["result"]["successful"], 3);
        assert_eq!(response_json["result"]["failed"].as_array().unwrap().len(), 0);
        assert_eq!(response_json["starting_vector_index"], 0);
        assert_eq!(response_json["ending_vector_index"], 2);
    }

    #[tokio::test]
    async fn test_bulk_upload_with_validation_errors() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", temp_path);

        let app = create_app();

        // Create bulk upload data with some invalid reviews
        let bulk_data = json!([
            {
                "title": "Valid review",
                "body": "This is a valid review.",
                "product_id": "prod_001",
                "rating": 5
            },
            {
                "title": "", // Invalid - empty title
                "body": "This review has empty title.",
                "product_id": "prod_002",
                "rating": 4
            },
            {
                "title": "Another valid review",
                "body": "This is another valid review.",
                "product_id": "prod_003",
                "rating": 3
            },
            {
                "title": "Invalid rating review",
                "body": "This review has invalid rating.",
                "product_id": "prod_004",
                "rating": 6 // Invalid rating
            }
        ]);

        let request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(bulk_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["result"]["total_processed"], 4);
        assert_eq!(response_json["result"]["successful"], 2); // Only 2 valid reviews
        assert_eq!(response_json["result"]["failed"].as_array().unwrap().len(), 2); // 2 failed reviews
        
        // Check that failed reviews have proper error information
        let failed_reviews = response_json["result"]["failed"].as_array().unwrap();
        assert!(failed_reviews[0]["error"].as_str().unwrap().contains("title"));
        assert!(failed_reviews[1]["error"].as_str().unwrap().contains("rating"));
    }

    #[tokio::test]
    async fn test_bulk_upload_jsonl_format() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", temp_path);

        let app = create_app();

        // Create bulk upload data as JSONL string
        let jsonl_data = r#"{"title": "JSONL Review 1", "body": "First review in JSONL format.", "product_id": "jsonl_001", "rating": 5}
{"title": "JSONL Review 2", "body": "Second review in JSONL format.", "product_id": "jsonl_002", "rating": 4}"#;

        let request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(json!(jsonl_data).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["result"]["total_processed"], 2);
        assert_eq!(response_json["result"]["successful"], 2);
        assert_eq!(response_json["result"]["failed"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_bulk_upload_empty_data() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", temp_path);

        let app = create_app();

        // Create empty bulk upload data
        let empty_data = json!([]);

        let request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(empty_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["error"], "validation_error");
        assert!(response_json["message"].as_str().unwrap().contains("No valid reviews found"));
    }

    #[tokio::test]
    async fn test_search_reviews_endpoint() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/search_test", temp_path));

        let app = create_app();

        // First, add some reviews to search through
        let reviews_to_add = vec![
            json!({
                "title": "Amazing smartphone",
                "body": "This phone has excellent camera quality and fast performance. Great value for money.",
                "product_id": "phone_001",
                "rating": 5
            }),
            json!({
                "title": "Good laptop",
                "body": "Decent performance for work. Battery life could be better but overall satisfied.",
                "product_id": "laptop_001", 
                "rating": 4
            }),
            json!({
                "title": "Poor quality headphones",
                "body": "Sound quality is terrible and they broke after one week. Not recommended.",
                "product_id": "headphones_001",
                "rating": 1
            })
        ];

        // Add reviews using bulk upload
        let bulk_request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(json!(reviews_to_add).to_string()))
            .unwrap();

        let bulk_response = app.clone().oneshot(bulk_request).await.unwrap();
        assert_eq!(bulk_response.status(), StatusCode::OK);

        // Now test search functionality
        let search_data = json!({
            "query": "camera quality",
            "limit": 10
        });

        let search_request = Request::builder()
            .method("POST")
            .uri("/search")
            .header("content-type", "application/json")
            .body(Body::from(search_data.to_string()))
            .unwrap();

        let search_response = app.oneshot(search_request).await.unwrap();

        assert_eq!(search_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(search_response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["query"], "camera quality");
        assert_eq!(response_json["search_type"], "text_similarity");
        
        let results = response_json["results"].as_array().unwrap();
        assert!(results.len() > 0, "Should find at least one matching review");
        
        // The smartphone review should be the top result due to "camera quality" match
        let top_result = &results[0];
        assert!(top_result["review"]["title"].as_str().unwrap().contains("smartphone"));
        assert!(top_result["similarity_score"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_search_reviews_validation_error() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/search_validation", temp_path));

        let app = create_app();

        // Create search request with empty query
        let invalid_search_data = json!({
            "query": "",
            "limit": 10
        });

        let request = Request::builder()
            .method("POST")
            .uri("/search")
            .header("content-type", "application/json")
            .body(Body::from(invalid_search_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["error"], "validation_error");
        assert!(response_json["message"].as_str().unwrap().contains("query"));
    }

    #[tokio::test]
    async fn test_search_reviews_no_results() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/search_no_results", temp_path));

        let app = create_app();

        // Search without adding any reviews first
        let search_data = json!({
            "query": "nonexistent product",
            "limit": 10
        });

        let request = Request::builder()
            .method("POST")
            .uri("/search")
            .header("content-type", "application/json")
            .body(Body::from(search_data.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["total_results"], 0);
        assert_eq!(response_json["results"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_search_reviews_ranking() {
        // Set up temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("DATA_DIR", format!("{}/search_ranking", temp_path));

        let app = create_app();

        // Add reviews with different relevance to query "fast performance"
        let reviews_to_add = vec![
            json!({
                "title": "Fast performance laptop", // Exact match in title
                "body": "This laptop delivers amazing speed and efficiency.",
                "product_id": "laptop_002",
                "rating": 5
            }),
            json!({
                "title": "Good computer",
                "body": "Has fast performance and great build quality.", // Exact match in body
                "product_id": "computer_001",
                "rating": 4
            }),
            json!({
                "title": "Slow device",
                "body": "This device is quite slow and not recommended.", // No match
                "product_id": "device_001",
                "rating": 2
            })
        ];

        // Add reviews using bulk upload
        let bulk_request = Request::builder()
            .method("POST")
            .uri("/reviews/bulk")
            .header("content-type", "application/json")
            .body(Body::from(json!(reviews_to_add).to_string()))
            .unwrap();

        let bulk_response = app.clone().oneshot(bulk_request).await.unwrap();
        assert_eq!(bulk_response.status(), StatusCode::OK);

        // Search for "fast performance"
        let search_data = json!({
            "query": "fast performance",
            "limit": 10
        });

        let search_request = Request::builder()
            .method("POST")
            .uri("/search")
            .header("content-type", "application/json")
            .body(Body::from(search_data.to_string()))
            .unwrap();

        let search_response = app.oneshot(search_request).await.unwrap();
        assert_eq!(search_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(search_response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let results = response_json["results"].as_array().unwrap();
        assert!(results.len() >= 2, "Should find at least 2 matching reviews");

        // Results should be ranked by similarity score (descending)
        let first_score = results[0]["similarity_score"].as_f64().unwrap();
        let second_score = results[1]["similarity_score"].as_f64().unwrap();
        assert!(first_score >= second_score, "Results should be ranked by similarity score");

        // The laptop with exact title match should have highest score
        let top_result = &results[0];
        assert!(top_result["review"]["title"].as_str().unwrap().contains("Fast performance"));
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = create_app();

        let request = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["status"], "healthy");
        assert_eq!(response_json["service"], "semantic-search-backend");
    }
}