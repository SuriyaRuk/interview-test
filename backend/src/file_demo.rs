use crate::models::*;
use crate::storage::*;
use std::env;

/// Demonstration of Requirements 4.2, 4.3, and 4.4 implementation
pub struct FileSystemDemo {
    data_paths: DataPaths,
    jsonl_storage: JsonlStorage,
}

impl FileSystemDemo {
    /// Initialize the file system demo with proper data directory structure
    pub fn new() -> Result<Self, AppError> {
        // Requirement 4.1: Create necessary data directory structure if it doesn't exist
        let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
        let data_paths = DataPaths::new(&data_dir);
        
        // Ensure directories exist
        data_paths.ensure_directories()?;
        
        let jsonl_storage = JsonlStorage::new(&data_paths.reviews_jsonl);
        
        Ok(Self {
            data_paths,
            jsonl_storage,
        })
    }
    
    /// Demonstrate Requirement 4.2: Use SPFresh binary index files for vector storage
    /// Note: This is a placeholder showing the structure - actual SPFresh integration comes later
    pub fn demonstrate_vector_storage(&self) -> Result<(), AppError> {
        println!("📁 Requirement 4.2: Vector Storage Structure");
        println!("Vector index file path: {:?}", self.data_paths.reviews_index);
        
        // Check if vector index file exists
        let (jsonl_exists, index_exists) = self.data_paths.files_exist();
        println!("JSONL file exists: {}", jsonl_exists);
        println!("Vector index file exists: {}", index_exists);
        
        // This demonstrates the file structure for SPFresh binary index
        println!("Vector index will store binary embeddings in: reviews.index");
        println!("Each vector corresponds to the same line number in reviews.jsonl");
        
        Ok(())
    }
    
    /// Demonstrate Requirement 4.3: Use JSONL format with one review per line
    pub fn demonstrate_jsonl_format(&self) -> Result<(), AppError> {
        println!("\n📄 Requirement 4.3: JSONL Format Implementation");
        
        // Create sample reviews to demonstrate JSONL format
        let sample_reviews = vec![
            ReviewData {
                title: "Great product!".to_string(),
                body: "This product exceeded my expectations. Great quality and fast delivery.".to_string(),
                product_id: "prod_123".to_string(),
                rating: 5,
            },
            ReviewData {
                title: "Good value".to_string(),
                body: "Decent product for the price. Would recommend to others.".to_string(),
                product_id: "prod_124".to_string(),
                rating: 4,
            },
            ReviewData {
                title: "Average experience".to_string(),
                body: "The product is okay but nothing special. Could be improved.".to_string(),
                product_id: "prod_125".to_string(),
                rating: 3,
            },
        ];
        
        // Convert to metadata and store in JSONL format
        let mut metadata_reviews = Vec::new();
        for (index, review_data) in sample_reviews.iter().enumerate() {
            let metadata = review_data.to_metadata(index)?;
            metadata_reviews.push(metadata);
        }
        
        // Demonstrate JSONL storage - one review per line
        self.jsonl_storage.append_reviews(&metadata_reviews)?;
        
        println!("✅ Stored {} reviews in JSONL format", metadata_reviews.len());
        println!("Each review is stored as one JSON object per line");
        
        // Demonstrate reading back the data
        let count = self.jsonl_storage.count_reviews()?;
        println!("Total reviews in JSONL file: {}", count);
        
        // Show sample of JSONL format
        if let Some(first_review) = self.jsonl_storage.get_review_by_index(0)? {
            println!("Sample JSONL entry:");
            println!("{}", serde_json::to_string_pretty(&first_review)?);
        }
        
        Ok(())
    }
    
    /// Demonstrate Requirement 4.4: Use zero-based index positioning for correlation
    pub fn demonstrate_index_correlation(&self) -> Result<(), AppError> {
        println!("\n🔗 Requirement 4.4: Zero-based Index Correlation");
        
        // Demonstrate that vector index correlates with JSONL line numbers
        let total_reviews = self.jsonl_storage.count_reviews()?;
        println!("Total reviews for correlation demo: {}", total_reviews);
        
        if total_reviews > 0 {
            // Show correlation between vector index and JSONL line position
            for i in 0..std::cmp::min(total_reviews, 3) {
                if let Some(review) = self.jsonl_storage.get_review_by_index(i)? {
                    println!("JSONL line {}: vector_index={}, review_id={}", 
                            i, review.vector_index, review.id);
                    println!("  ↳ Vector at index {} in reviews.index corresponds to this review", 
                            review.vector_index);
                }
            }
            
            // Demonstrate batch retrieval using indices
            let indices = vec![0, 2]; // Get first and third reviews
            let batch_reviews = self.jsonl_storage.get_reviews_by_indices(&indices)?;
            
            println!("\nBatch retrieval demonstration:");
            for (i, review_opt) in batch_reviews.iter().enumerate() {
                if let Some(review) = review_opt {
                    println!("Index {}: Retrieved review '{}' (vector_index: {})", 
                            indices[i], review.title, review.vector_index);
                }
            }
        }
        
        Ok(())
    }
    
    /// Demonstrate file integrity and error handling (Requirement 4.5)
    pub fn demonstrate_error_handling(&self) -> Result<(), AppError> {
        println!("\n🛡️  File Integrity and Error Handling");
        
        // Validate JSONL file integrity
        let validation_result = self.jsonl_storage.validate_file()?;
        
        println!("File validation results:");
        println!("  Valid: {}", validation_result.is_valid);
        println!("  Total lines: {}", validation_result.total_lines);
        println!("  Valid lines: {}", validation_result.valid_lines);
        
        if !validation_result.errors.is_empty() {
            println!("  Errors found:");
            for error in &validation_result.errors {
                println!("    - {}", error);
            }
        } else {
            println!("  ✅ No validation errors found");
        }
        
        // Demonstrate graceful handling of missing files
        let non_existent_storage = JsonlStorage::new("non_existent_file.jsonl");
        let count = non_existent_storage.count_reviews()?;
        println!("Count from non-existent file (graceful handling): {}", count);
        
        Ok(())
    }
    
    /// Demonstrate concurrent access safety
    pub fn demonstrate_concurrent_safety(&self) -> Result<(), AppError> {
        println!("\n🔒 Concurrent Access Safety");
        
        // Demonstrate file locking for concurrent operations
        println!("Acquiring file lock for safe concurrent access...");
        let _lock = FileLock::acquire(&self.data_paths.lock_file)?;
        println!("✅ File lock acquired successfully");
        
        // The lock will be automatically released when _lock goes out of scope
        println!("File lock will be released automatically when operation completes");
        
        Ok(())
    }
    
    /// Run complete demonstration of all file system requirements
    pub fn run_complete_demo(&self) -> Result<(), AppError> {
        println!("🚀 File System Data Models Demonstration");
        println!("Demonstrating Requirements 4.2, 4.3, and 4.4\n");
        
        self.demonstrate_vector_storage()?;
        self.demonstrate_jsonl_format()?;
        self.demonstrate_index_correlation()?;
        self.demonstrate_error_handling()?;
        self.demonstrate_concurrent_safety()?;
        
        println!("\n✅ All file system requirements demonstrated successfully!");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    #[test]
    fn test_complete_file_system_demo() {
        // Use temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        
        // Set temporary data directory
        env::set_var("DATA_DIR", temp_path);
        
        // Run the complete demonstration
        let demo = FileSystemDemo::new().unwrap();
        demo.run_complete_demo().unwrap();
        
        // Verify files were created
        let (jsonl_exists, _) = demo.data_paths.files_exist();
        assert!(jsonl_exists, "JSONL file should be created");
        
        // Verify data integrity
        let validation = demo.jsonl_storage.validate_file().unwrap();
        assert!(validation.is_valid, "JSONL file should be valid");
        assert!(validation.valid_lines > 0, "Should have valid review entries");
    }
    
    #[test]
    fn test_requirement_4_2_vector_storage_structure() {
        let temp_dir = TempDir::new().unwrap();
        let data_paths = DataPaths::new(temp_dir.path());
        data_paths.ensure_directories().unwrap();
        
        // Verify vector index file path structure
        assert!(data_paths.reviews_index.to_string_lossy().ends_with("reviews.index"));
        assert!(data_paths.reviews_jsonl.to_string_lossy().ends_with("reviews.jsonl"));
        assert!(data_paths.lock_file.to_string_lossy().ends_with(".lock"));
    }
    
    #[test]
    fn test_requirement_4_3_jsonl_format() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("test_reviews.jsonl");
        let storage = JsonlStorage::new(&jsonl_path);
        
        // Create test review
        let review_data = ReviewData {
            title: "Test Review".to_string(),
            body: "This is a test review for JSONL format verification.".to_string(),
            product_id: "test_prod".to_string(),
            rating: 4,
        };
        
        let metadata = review_data.to_metadata(0).unwrap();
        storage.append_review(&metadata).unwrap();
        
        // Verify JSONL format - one review per line
        let content = std::fs::read_to_string(&jsonl_path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines.len(), 1, "Should have exactly one line for one review");
        
        // Verify it's valid JSON
        let parsed: ReviewMetadata = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.title, "Test Review");
    }
    
    #[test]
    fn test_requirement_4_4_zero_based_index_correlation() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("test_reviews.jsonl");
        let storage = JsonlStorage::new(&jsonl_path);
        
        // Create multiple reviews with specific vector indices
        let reviews = vec![
            ReviewData {
                title: "Review 0".to_string(),
                body: "First review".to_string(),
                product_id: "prod_0".to_string(),
                rating: 5,
            }.to_metadata(0).unwrap(),
            ReviewData {
                title: "Review 1".to_string(),
                body: "Second review".to_string(),
                product_id: "prod_1".to_string(),
                rating: 4,
            }.to_metadata(1).unwrap(),
            ReviewData {
                title: "Review 2".to_string(),
                body: "Third review".to_string(),
                product_id: "prod_2".to_string(),
                rating: 3,
            }.to_metadata(2).unwrap(),
        ];
        
        storage.append_reviews(&reviews).unwrap();
        
        // Verify zero-based index correlation
        for i in 0..3 {
            let retrieved = storage.get_review_by_index(i).unwrap().unwrap();
            assert_eq!(retrieved.vector_index, i, 
                      "Vector index should match JSONL line position");
            assert_eq!(retrieved.title, format!("Review {}", i));
        }
        
        // Test batch retrieval with specific indices
        let indices = vec![0, 2];
        let batch = storage.get_reviews_by_indices(&indices).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].as_ref().unwrap().vector_index, 0);
        assert_eq!(batch[1].as_ref().unwrap().vector_index, 2);
    }
}