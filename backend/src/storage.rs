use crate::models::*;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write, BufWriter};
use serde_json;

/// Data directory structure constants
pub struct DataPaths {
    pub data_dir: PathBuf,
    pub reviews_jsonl: PathBuf,
    pub reviews_index: PathBuf,
    pub lock_file: PathBuf,
}

impl DataPaths {
    /// Create new DataPaths with the given data directory
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        
        Self {
            reviews_jsonl: data_dir.join("reviews.jsonl"),
            reviews_index: data_dir.join("reviews.index"),
            lock_file: data_dir.join(".lock"),
            data_dir,
        }
    }
    
    /// Ensure all necessary directories exist
    pub fn ensure_directories(&self) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
    
    /// Check if data files exist
    pub fn files_exist(&self) -> (bool, bool) {
        (
            self.reviews_jsonl.exists(),
            self.reviews_index.exists(),
        )
    }
}

/// JSONL file operations for ReviewMetadata
pub struct JsonlStorage {
    file_path: PathBuf,
}

impl JsonlStorage {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }
    
    /// Append a single ReviewMetadata to the JSONL file
    pub fn append_review(&self, review: &ReviewMetadata) -> Result<(), AppError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
            
        let json_line = serde_json::to_string(review)?;
        writeln!(file, "{}", json_line)?;
        file.flush()?;
        
        Ok(())
    }
    
    /// Append multiple ReviewMetadata to the JSONL file
    pub fn append_reviews(&self, reviews: &[ReviewMetadata]) -> Result<(), AppError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
            
        let mut writer = BufWriter::new(&mut file);
        
        for review in reviews {
            let json_line = serde_json::to_string(review)?;
            writeln!(writer, "{}", json_line)?;
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Read a specific review by line index (0-based)
    pub fn get_review_by_index(&self, index: usize) -> Result<Option<ReviewMetadata>, AppError> {
        if !self.file_path.exists() {
            return Ok(None);
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        for (line_index, line) in reader.lines().enumerate() {
            if line_index == index {
                let line = line?;
                if line.trim().is_empty() {
                    return Ok(None);
                }
                let review: ReviewMetadata = serde_json::from_str(&line)?;
                return Ok(Some(review));
            }
        }
        
        Ok(None)
    }
    
    /// Read multiple reviews by their line indices
    pub fn get_reviews_by_indices(&self, indices: &[usize]) -> Result<Vec<Option<ReviewMetadata>>, AppError> {
        if !self.file_path.exists() {
            return Ok(vec![None; indices.len()]);
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        let mut results = vec![None; indices.len()];
        let mut target_indices: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
        
        // Group result indices by line index for efficient lookup
        for (result_idx, &line_idx) in indices.iter().enumerate() {
            target_indices.entry(line_idx).or_insert_with(Vec::new).push(result_idx);
        }
        
        for (line_index, line) in reader.lines().enumerate() {
            if let Some(result_indices) = target_indices.get(&line_index) {
                let line = line?;
                if !line.trim().is_empty() {
                    let review: ReviewMetadata = serde_json::from_str(&line)?;
                    for &result_idx in result_indices {
                        results[result_idx] = Some(review.clone());
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Count total number of reviews in the file
    pub fn count_reviews(&self) -> Result<usize, AppError> {
        if !self.file_path.exists() {
            return Ok(0);
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    /// Read all reviews from the file (use with caution for large files)
    pub fn read_all_reviews(&self) -> Result<Vec<ReviewMetadata>, AppError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        let mut reviews = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let review: ReviewMetadata = serde_json::from_str(&line)?;
                reviews.push(review);
            }
        }
        
        Ok(reviews)
    }
    
    /// Validate the integrity of the JSONL file
    pub fn validate_file(&self) -> Result<ValidationResult, AppError> {
        if !self.file_path.exists() {
            return Ok(ValidationResult {
                is_valid: true,
                total_lines: 0,
                valid_lines: 0,
                errors: Vec::new(),
            });
        }
        
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        let mut total_lines = 0;
        let mut valid_lines = 0;
        let mut errors = Vec::new();
        
        for (line_number, line) in reader.lines().enumerate() {
            total_lines += 1;
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<ReviewMetadata>(&line) {
                Ok(_) => valid_lines += 1,
                Err(e) => errors.push(ValidationError::InvalidValue {
                    field: format!("line_{}", line_number + 1),
                    reason: e.to_string(),
                }),
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            total_lines,
            valid_lines,
            errors,
        })
    }
}

/// Result of file validation
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub total_lines: usize,
    pub valid_lines: usize,
    pub errors: Vec<ValidationError>,
}

/// File locking utilities for concurrent access
pub struct FileLock {
    lock_file: PathBuf,
    _lock: File,
}

impl FileLock {
    pub fn acquire<P: AsRef<Path>>(lock_file: P) -> Result<Self, AppError> {
        use fs2::FileExt;
        
        let lock_file = lock_file.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_file)?;
            
        file.lock_exclusive().map_err(|e| AppError::Concurrency {
            message: format!("Failed to acquire file lock: {}", e),
        })?;
        
        Ok(Self {
            lock_file,
            _lock: file,
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        use fs2::FileExt;
        let _ = self._lock.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::Utc;

    fn create_test_review(id: &str, vector_index: usize) -> ReviewMetadata {
        ReviewMetadata {
            id: id.to_string(),
            title: "Test Review".to_string(),
            body: "This is a test review body.".to_string(),
            product_id: "test_product".to_string(),
            rating: 5,
            timestamp: Utc::now(),
            vector_index,
        }
    }

    #[test]
    fn test_jsonl_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let jsonl_path = temp_dir.path().join("test_reviews.jsonl");
        let storage = JsonlStorage::new(&jsonl_path);

        // Test append single review
        let review1 = create_test_review("rev_001", 0);
        storage.append_review(&review1).unwrap();

        // Test append multiple reviews
        let reviews = vec![
            create_test_review("rev_002", 1),
            create_test_review("rev_003", 2),
        ];
        storage.append_reviews(&reviews).unwrap();

        // Test count
        assert_eq!(storage.count_reviews().unwrap(), 3);

        // Test get by index
        let retrieved = storage.get_review_by_index(0).unwrap().unwrap();
        assert_eq!(retrieved.id, "rev_001");

        // Test get multiple by indices
        let retrieved_multiple = storage.get_reviews_by_indices(&[0, 2]).unwrap();
        assert_eq!(retrieved_multiple.len(), 2);
        assert_eq!(retrieved_multiple[0].as_ref().unwrap().id, "rev_001");
        assert_eq!(retrieved_multiple[1].as_ref().unwrap().id, "rev_003");

        // Test validation
        let validation = storage.validate_file().unwrap();
        assert!(validation.is_valid);
        assert_eq!(validation.valid_lines, 3);
    }

    #[test]
    fn test_data_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = DataPaths::new(temp_dir.path());

        // Test directory creation
        paths.ensure_directories().unwrap();
        assert!(paths.data_dir.exists());

        // Test file existence check
        let (jsonl_exists, index_exists) = paths.files_exist();
        assert!(!jsonl_exists);
        assert!(!index_exists);
    }
}