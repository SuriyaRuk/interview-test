use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window, Request, RequestInit, RequestMode, Response, Headers, HtmlInputElement, HtmlTextAreaElement, HtmlSelectElement, FileReader, HtmlFormElement};
use js_sys::Promise;
use serde::{Deserialize, Serialize};

// API Configuration - Use environment variable or fallback to default
const API_BASE_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://192.168.1.2:8000",
};

// API Models based on README.md specification
#[derive(Serialize, Deserialize)]
struct CreateReviewRequest {
    title: String,
    body: String,
    product_id: String,
    rating: u8,
}

#[derive(Serialize, Deserialize)]
struct CreateReviewResponse {
    success: bool,
    message: String,
    review_id: String,
    vector_index: u32,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct SearchRequest {
    query: String,
    limit: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct SearchResponse {
    success: bool,
    query: String,
    results: Vec<SearchResult>,
    total_results: u32,
    limit: u32,
    search_type: String,
}

#[derive(Serialize, Deserialize)]
struct SearchResult {
    review: ReviewData,
    similarity_score: f64,
}

#[derive(Serialize, Deserialize)]
struct ReviewData {
    id: String,
    title: String,
    body: String,
    product_id: String,
    rating: u8,
    timestamp: String,
    vector_index: u32,
}

#[derive(Serialize, Deserialize)]
struct BulkUploadResponse {
    success: bool,
    message: String,
    result: BulkUploadResult,
    starting_vector_index: u32,
    ending_vector_index: u32,
}

#[derive(Serialize, Deserialize)]
struct BulkUploadResult {
    total_processed: u32,
    successful: u32,
    failed: Vec<BulkUploadError>,
}

#[derive(Serialize, Deserialize)]
struct BulkUploadError {
    line_number: u32,
    error: String,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ApiError {
    error: String,
    message: String,
    details: Option<serde_json::Value>,
    timestamp: String,
}

/// Entry point for the WebAssembly module
/// This function is called from JavaScript to initialize and start the application
#[wasm_bindgen]
pub fn main() {
    // Set up panic hook for better error reporting in the browser console
    console_error_panic_hook::set_once();
    
    // Log successful initialization (using web_sys console directly)
    console::log_1(&"🚀 WebAssembly module initialized successfully".into());
    
    // Create and mount the application
    if let Err(e) = create_app() {
        console::error_1(&format!("Failed to create app: {:?}", e).into());
        return;
    }
    
    // Log successful mounting
    console::log_1(&"✅ Application mounted to DOM".into());
}

/// Alternative entry point using wasm-bindgen start attribute
/// This can be used if automatic initialization is preferred
#[wasm_bindgen(start)]
pub fn start() {
    main();
}

/// Create and mount the Semantic Search Platform application
fn create_app() -> Result<(), JsValue> {
    let window = window().ok_or("No global window exists")?;
    let document = window.document().ok_or("Should have a document on window")?;
    
    // Find the app container
    let app_container = document
        .get_element_by_id("app")
        .ok_or("Should have an element with id 'app'")?;
    
    // Create the main application HTML
    let app_html = r#"
        <div class="home-page">
            <header class="header">
                <h1>🔍 Semantic Search Platform</h1>
                <p class="subtitle">Search product reviews using natural language</p>
            </header>
            
            <div class="main-content">
                <div class="section">
                    <h2>Add Reviews</h2>
                    <div class="component-placeholder">
                        <form id="review-form">
                            <div class="form-group">
                                <label for="product-name">Product Name:</label>
                                <input type="text" id="product-name" name="product-name" required>
                            </div>
                            <div class="form-group">
                                <label for="review-text">Review:</label>
                                <textarea id="review-text" name="review-text" rows="4" required></textarea>
                            </div>
                            <div class="form-group">
                                <label for="rating">Rating:</label>
                                <select id="rating" name="rating" required>
                                    <option value="">Select rating</option>
                                    <option value="1">1 Star</option>
                                    <option value="2">2 Stars</option>
                                    <option value="3">3 Stars</option>
                                    <option value="4">4 Stars</option>
                                    <option value="5">5 Stars</option>
                                </select>
                            </div>
                            <button type="submit">Add Review</button>
                        </form>
                    </div>
                </div>
                
                <div class="section">
                    <h2>Bulk Upload</h2>
                    <div class="component-placeholder">
                        <div id="bulk-upload">
                            <input type="file" id="file-input" accept=".csv,.json" multiple>
                            <button id="upload-btn">Upload Files</button>
                            <div id="upload-status"></div>
                        </div>
                    </div>
                </div>
                
                <div class="section">
                    <h2>Search Reviews</h2>
                    <div class="component-placeholder">
                        <div id="search-interface">
                            <div class="search-form">
                                <input type="text" id="search-input" placeholder="Search reviews using natural language...">
                                <button id="search-btn">Search</button>
                            </div>
                            <div id="search-results"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    "#;
    
    // Set the HTML content
    app_container.set_inner_html(app_html);
    
    // Add event listeners
    setup_event_listeners(&document)?;
    
    console::log_1(&"✅ Application HTML created and event listeners attached".into());
    
    Ok(())
}

/// HTTP client functions for API communication
async fn make_api_request(method: &str, endpoint: &str, body: Option<String>) -> Result<Response, JsValue> {
    let url = format!("{}{}", API_BASE_URL, endpoint);
    
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    
    // Set headers
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.set_headers(&headers);
    
    // Set body if provided
    if let Some(body_str) = body {
        opts.set_body(&JsValue::from_str(&body_str));
    }
    
    let request = Request::new_with_str_and_init(&url, &opts)?;
    
    let window = window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    
    Ok(resp)
}

/// Create a new review
async fn create_review(request: CreateReviewRequest) -> Result<CreateReviewResponse, JsValue> {
    let body = serde_json::to_string(&request).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let response = make_api_request("POST", "/reviews", Some(body)).await?;
    
    if !response.ok() {
        let error_text = JsFuture::from(response.text()?).await?;
        return Err(JsValue::from_str(&format!("API Error: {}", error_text.as_string().unwrap_or_default())));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let result: CreateReviewResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    Ok(result)
}

/// Search reviews
async fn search_reviews(request: SearchRequest) -> Result<SearchResponse, JsValue> {
    let body = serde_json::to_string(&request).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let response = make_api_request("POST", "/search", Some(body)).await?;
    
    if !response.ok() {
        let error_text = JsFuture::from(response.text()?).await?;
        return Err(JsValue::from_str(&format!("API Error: {}", error_text.as_string().unwrap_or_default())));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let result: SearchResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    Ok(result)
}

/// Bulk upload reviews
async fn bulk_upload_reviews(data: String) -> Result<BulkUploadResponse, JsValue> {
    let response = make_api_request("POST", "/reviews/bulk", Some(data)).await?;
    
    if !response.ok() {
        let error_text = JsFuture::from(response.text()?).await?;
        return Err(JsValue::from_str(&format!("API Error: {}", error_text.as_string().unwrap_or_default())));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let result: BulkUploadResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    Ok(result)
}

/// Display success message
fn show_message(element_id: &str, message: &str, is_error: bool) {
    if let Some(element) = window().unwrap().document().unwrap().get_element_by_id(element_id) {
        let class = if is_error { "error-message" } else { "success-message" };
        element.set_inner_html(&format!(r#"<div class="{}">{}</div>"#, class, message));
    }
}

/// Display search results
fn display_search_results(results: Vec<SearchResult>) {
    let document = window().unwrap().document().unwrap();
    if let Some(results_div) = document.get_element_by_id("search-results") {
        if results.is_empty() {
            results_div.set_inner_html(r#"
                <div class="no-results">
                    <p>No reviews found matching your search.</p>
                    <p>Try different keywords or check your spelling.</p>
                </div>
            "#);
            return;
        }
        
        let mut html = String::from(r#"<h3>Search Results</h3><div class="results-list">"#);
        
        for result in results {
            let stars = "★".repeat(result.review.rating as usize) + &"☆".repeat(5 - result.review.rating as usize);
            html.push_str(&format!(r#"
                <div class="result-item">
                    <div class="result-header">
                        <h4 class="result-title">{}</h4>
                        <div class="result-meta">
                            <span class="similarity-score">{:.1}% match</span>
                            <span class="rating">{}</span>
                        </div>
                    </div>
                    <p class="result-body">{}</p>
                    <div class="result-footer">
                        <span class="product-id">Product: {}</span>
                        <span class="timestamp">{}</span>
                    </div>
                </div>
            "#, 
                result.review.title,
                result.similarity_score * 100.0,
                stars,
                result.review.body,
                result.review.product_id,
                result.review.timestamp
            ));
        }
        
        html.push_str("</div>");
        results_div.set_inner_html(&html);
    }
}

/// Set up event listeners for the application
fn setup_event_listeners(document: &web_sys::Document) -> Result<(), JsValue> {
    // Review form submission
    if let Some(form) = document.get_element_by_id("review-form") {
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            event.prevent_default();
            console::log_1(&"Review form submitted".into());
            
            wasm_bindgen_futures::spawn_local(async move {
                let document = window().unwrap().document().unwrap();
                
                // Get form values
                let product_name = document.get_element_by_id("product-name")
                    .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                    .map(|input| input.value())
                    .unwrap_or_default();
                
                let review_text = document.get_element_by_id("review-text")
                    .and_then(|e| e.dyn_into::<HtmlTextAreaElement>().ok())
                    .map(|textarea| textarea.value())
                    .unwrap_or_default();
                
                let rating_str = document.get_element_by_id("rating")
                    .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
                    .map(|select| select.value())
                    .unwrap_or_default();
                
                // Validate inputs
                if product_name.trim().is_empty() || review_text.trim().is_empty() || rating_str.is_empty() {
                    show_message("review-form", "Please fill in all fields", true);
                    return;
                }
                
                let rating = match rating_str.parse::<u8>() {
                    Ok(r) if r >= 1 && r <= 5 => r,
                    _ => {
                        show_message("review-form", "Please select a valid rating", true);
                        return;
                    }
                };
                
                // Create review request
                let request = CreateReviewRequest {
                    title: product_name.clone(),
                    body: review_text,
                    product_id: product_name,
                    rating,
                };
                
                // Show loading state
                if let Some(button) = document.get_element_by_id("review-form")
                    .and_then(|form| form.query_selector("button[type='submit']").ok().flatten()) {
                    button.set_text_content(Some("Adding Review..."));
                }
                
                // Make API call
                match create_review(request).await {
                    Ok(response) => {
                        console::log_1(&format!("Review created: {}", response.message).into());
                        show_message("review-form", &format!("✅ {}", response.message), false);
                        
                        // Clear form
                        if let Some(form) = document.get_element_by_id("review-form")
                            .and_then(|e| e.dyn_into::<HtmlFormElement>().ok()) {
                            form.reset();
                        }
                    }
                    Err(error) => {
                        console::error_1(&format!("Failed to create review: {:?}", error).into());
                        show_message("review-form", "❌ Failed to add review. Please try again.", true);
                    }
                }
                
                // Reset button text
                if let Some(button) = document.get_element_by_id("review-form")
                    .and_then(|form| form.query_selector("button[type='submit']").ok().flatten()) {
                    button.set_text_content(Some("Add Review"));
                }
            });
        }) as Box<dyn FnMut(_)>);
        
        form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget(); // Keep the closure alive
    }
    
    // Search button
    if let Some(search_btn) = document.get_element_by_id("search-btn") {
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            console::log_1(&"Search button clicked".into());
            
            wasm_bindgen_futures::spawn_local(async move {
                let document = window().unwrap().document().unwrap();
                
                // Get search query
                let query = document.get_element_by_id("search-input")
                    .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                    .map(|input| input.value())
                    .unwrap_or_default();
                
                if query.trim().is_empty() {
                    show_message("search-results", "Please enter a search query", true);
                    return;
                }
                
                // Show loading state
                if let Some(button) = document.get_element_by_id("search-btn") {
                    button.set_text_content(Some("Searching..."));
                }
                
                if let Some(results_div) = document.get_element_by_id("search-results") {
                    results_div.set_inner_html("<p>🔍 Searching reviews...</p>");
                }
                
                // Create search request
                let request = SearchRequest {
                    query: query.trim().to_string(),
                    limit: Some(10),
                };
                
                // Make API call
                match search_reviews(request).await {
                    Ok(response) => {
                        console::log_1(&format!("Search completed: {} results", response.total_results).into());
                        display_search_results(response.results);
                    }
                    Err(error) => {
                        console::error_1(&format!("Search failed: {:?}", error).into());
                        show_message("search-results", "❌ Search failed. Please try again.", true);
                    }
                }
                
                // Reset button text
                if let Some(button) = document.get_element_by_id("search-btn") {
                    button.set_text_content(Some("Search"));
                }
            });
        }) as Box<dyn FnMut(_)>);
        
        search_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget(); // Keep the closure alive
    }
    
    // Upload button
    if let Some(upload_btn) = document.get_element_by_id("upload-btn") {
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            console::log_1(&"Upload button clicked".into());
            
            wasm_bindgen_futures::spawn_local(async move {
                let document = window().unwrap().document().unwrap();
                
                // Get selected files
                let files = document.get_element_by_id("file-input")
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .and_then(|input| input.files());
                
                if files.as_ref().map_or(true, |f| f.length() == 0) {
                    show_message("upload-status", "Please select files to upload", true);
                    return;
                }
                
                // Show loading state
                if let Some(button) = document.get_element_by_id("upload-btn") {
                    button.set_text_content(Some("Uploading..."));
                }
                
                show_message("upload-status", "📤 Processing files...", false);
                
                // Process each file
                if let Some(files) = files {
                    for i in 0..files.length() {
                        if let Some(file) = files.get(i) {
                            let file_name = file.name();
                            console::log_1(&format!("Processing file: {}", file_name).into());
                            
                            // Read file content
                            let file_reader = FileReader::new().unwrap();
                            let file_reader_clone = file_reader.clone();
                            
                            let promise = Promise::new(&mut |resolve, _reject| {
                                let file_reader_for_closure = file_reader_clone.clone();
                                let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                                    if let Ok(result) = file_reader_for_closure.result() {
                                        resolve.call1(&JsValue::NULL, &result).unwrap();
                                    }
                                }) as Box<dyn FnMut(_)>);
                                
                                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                onload.forget();
                            });
                            
                            file_reader.read_as_text(&file).unwrap();
                            
                            match JsFuture::from(promise).await {
                                Ok(content) => {
                                    let content_str = content.as_string().unwrap_or_default();
                                    
                                    // Make bulk upload API call
                                    match bulk_upload_reviews(content_str).await {
                                        Ok(response) => {
                                            console::log_1(&format!("Bulk upload completed: {}", response.message).into());
                                            show_message("upload-status", &format!("✅ {}", response.message), false);
                                        }
                                        Err(error) => {
                                            console::error_1(&format!("Bulk upload failed: {:?}", error).into());
                                            show_message("upload-status", &format!("❌ Failed to upload {}", file_name), true);
                                        }
                                    }
                                }
                                Err(error) => {
                                    console::error_1(&format!("Failed to read file: {:?}", error).into());
                                    show_message("upload-status", &format!("❌ Failed to read {}", file_name), true);
                                }
                            }
                        }
                    }
                }
                
                // Reset button text
                if let Some(button) = document.get_element_by_id("upload-btn") {
                    button.set_text_content(Some("Upload Files"));
                }
            });
        }) as Box<dyn FnMut(_)>);
        
        upload_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget(); // Keep the closure alive
    }
    
    Ok(())
}