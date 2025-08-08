use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Core review data structure for input
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewData {
    pub title: String,
    pub body: String,
    pub product_id: String,
    pub rating: u8, // 1-5 scale
}

/// Review metadata stored in JSONL file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub id: String,
    pub title: String,
    pub body: String,
    pub product_id: String,
    pub rating: u8,
    pub timestamp: DateTime<Utc>,
    pub vector_index: usize,
}

/// Search result with similarity score
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub review: ReviewMetadata,
    pub similarity_score: f32,
}

/// Search request structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>, // Default: 10
}

/// Bulk upload result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkUploadResult {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: Vec<BulkError>,
}

/// Individual bulk upload error
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkError {
    pub line_number: usize,
    pub error: String,
    pub data: Option<serde_json::Value>,
}

/// Standard API error response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid field value: {field} - {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Field too short: {field} must be at least {min_length} characters")]
    TooShort { field: String, min_length: usize },

    #[error("Field too long: {field} must be at most {max_length} characters")]
    TooLong { field: String, max_length: usize },

    #[error("Invalid rating: must be between 1 and 5")]
    InvalidRating,
}

/// Application errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("File operation error: {0}")]
    FileOperation(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UUID generation error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Embedding generation error: {message}")]
    Embedding { message: String },

    #[error("Vector search error: {message}")]
    VectorSearch { message: String },

    #[error("Concurrency error: {message}")]
    Concurrency { message: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },
}

impl ReviewData {
    /// Validate review data according to requirements
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check required fields
        if self.title.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "title".to_string(),
            });
        }

        if self.body.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "body".to_string(),
            });
        }

        if self.product_id.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "product_id".to_string(),
            });
        }

        // Check field lengths
        if self.title.len() < 3 {
            return Err(ValidationError::TooShort {
                field: "title".to_string(),
                min_length: 3,
            });
        }

        if self.title.len() > 200 {
            return Err(ValidationError::TooLong {
                field: "title".to_string(),
                max_length: 200,
            });
        }

        if self.body.len() < 10 {
            return Err(ValidationError::TooShort {
                field: "body".to_string(),
                min_length: 10,
            });
        }

        if self.body.len() > 2000 {
            return Err(ValidationError::TooLong {
                field: "body".to_string(),
                max_length: 2000,
            });
        }

        if self.product_id.len() > 100 {
            return Err(ValidationError::TooLong {
                field: "product_id".to_string(),
                max_length: 100,
            });
        }

        // Check rating range
        if self.rating < 1 || self.rating > 5 {
            return Err(ValidationError::InvalidRating);
        }

        Ok(())
    }

    /// Convert to ReviewMetadata with generated ID and timestamp
    pub fn to_metadata(&self, vector_index: usize) -> Result<ReviewMetadata, AppError> {
        self.validate()?;

        Ok(ReviewMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            title: self.title.clone(),
            body: self.body.clone(),
            product_id: self.product_id.clone(),
            rating: self.rating,
            timestamp: Utc::now(),
            vector_index,
        })
    }
}

impl SearchRequest {
    /// Validate search request
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.query.trim().is_empty() {
            return Err(ValidationError::MissingField {
                field: "query".to_string(),
            });
        }

        if self.query.len() > 500 {
            return Err(ValidationError::TooLong {
                field: "query".to_string(),
                max_length: 500,
            });
        }

        if let Some(limit) = self.limit {
            if limit == 0 || limit > 100 {
                return Err(ValidationError::InvalidValue {
                    field: "limit".to_string(),
                    reason: "must be between 1 and 100".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get the limit with default value
    pub fn get_limit(&self) -> usize {
        self.limit.unwrap_or(10)
    }
}

impl From<AppError> for ErrorResponse {
    fn from(error: AppError) -> Self {
        let (error_type, message, details) = match &error {
            AppError::Validation(validation_error) => (
                "validation_error".to_string(),
                validation_error.to_string(),
                None,
            ),
            AppError::FileOperation(io_error) => (
                "file_operation_error".to_string(),
                "File operation failed".to_string(),
                Some(serde_json::json!({ "io_error": io_error.to_string() })),
            ),
            AppError::Serialization(serde_error) => (
                "serialization_error".to_string(),
                "Data serialization failed".to_string(),
                Some(serde_json::json!({ "serde_error": serde_error.to_string() })),
            ),
            AppError::Embedding { message } => {
                ("embedding_error".to_string(), message.clone(), None)
            }
            AppError::VectorSearch { message } => {
                ("vector_search_error".to_string(), message.clone(), None)
            }
            AppError::Concurrency { message } => {
                ("concurrency_error".to_string(), message.clone(), None)
            }
            AppError::Internal { message } => ("internal_error".to_string(), message.clone(), None),
            _ => ("unknown_error".to_string(), error.to_string(), None),
        };

        ErrorResponse {
            error: error_type,
            message,
            details,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_data_validation() {
        // Valid review
        let valid_review = ReviewData {
            title: "Great product".to_string(),
            body: "This is a great product that I really enjoyed using.".to_string(),
            product_id: "prod_123".to_string(),
            rating: 5,
        };
        assert!(valid_review.validate().is_ok());

        // Missing title
        let invalid_review = ReviewData {
            title: "".to_string(),
            body: "This is a great product.".to_string(),
            product_id: "prod_123".to_string(),
            rating: 5,
        };
        assert!(invalid_review.validate().is_err());

        // Invalid rating
        let invalid_rating = ReviewData {
            title: "Great product".to_string(),
            body: "This is a great product.".to_string(),
            product_id: "prod_123".to_string(),
            rating: 6,
        };
        assert!(invalid_rating.validate().is_err());
    }

    #[test]
    fn test_search_request_validation() {
        // Valid search
        let valid_search = SearchRequest {
            query: "great product".to_string(),
            limit: Some(10),
        };
        assert!(valid_search.validate().is_ok());

        // Empty query
        let invalid_search = SearchRequest {
            query: "".to_string(),
            limit: Some(10),
        };
        assert!(invalid_search.validate().is_err());

        // Invalid limit
        let invalid_limit = SearchRequest {
            query: "great product".to_string(),
            limit: Some(0),
        };
        assert!(invalid_limit.validate().is_err());
    }
}
