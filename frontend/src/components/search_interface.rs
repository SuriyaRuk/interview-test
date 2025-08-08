use leptos::ev::SubmitEvent;
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub review: ReviewMetadata,
    pub similarity_score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub id: String,
    pub title: String,
    pub body: String,
    pub product_id: String,
    pub rating: u8,
    pub timestamp: String,
    pub vector_index: usize,
}

#[component]
pub fn SearchInterface() -> impl IntoView {
    let (query, set_query) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal(Vec::<SearchResult>::new());
    let (is_searching, set_is_searching) = create_signal(false);
    let (error, set_error) = create_signal(String::new());
    let (no_results, set_no_results) = create_signal(false);

    let search_reviews = create_action(move |query: &String| {
        let query = query.clone();
        async move {
            set_is_searching.set(true);
            set_error.set(String::new());
            set_no_results.set(false);
            set_search_results.set(Vec::new());

            // TODO: Replace with actual API call
            // For now, just simulate search results
            gloo_timers::future::TimeoutFuture::new(1500).await;

            // Mock search results
            let mock_results = vec![
                SearchResult {
                    review: ReviewMetadata {
                        id: "rev_001".to_string(),
                        title: "Amazing product!".to_string(),
                        body: "This product exceeded my expectations. Great quality and fast delivery.".to_string(),
                        product_id: "prod_123".to_string(),
                        rating: 5,
                        timestamp: "2024-01-15T10:30:00Z".to_string(),
                        vector_index: 0,
                    },
                    similarity_score: 0.95,
                },
                SearchResult {
                    review: ReviewMetadata {
                        id: "rev_002".to_string(),
                        title: "Good value".to_string(),
                        body: "Decent product for the price. Would recommend to others.".to_string(),
                        product_id: "prod_124".to_string(),
                        rating: 4,
                        timestamp: "2024-01-16T14:20:00Z".to_string(),
                        vector_index: 1,
                    },
                    similarity_score: 0.78,
                },
            ];

            set_is_searching.set(false);

            if query.trim().is_empty() {
                set_error.set("Please enter a search query".to_string());
            } else if mock_results.is_empty() {
                set_no_results.set(true);
            } else {
                set_search_results.set(mock_results);
            }
        }
    });

    let on_search = move |ev: SubmitEvent| {
        ev.prevent_default();
        let query_text = query.get().trim().to_string();

        if query_text.is_empty() {
            set_error.set("Please enter a search query".to_string());
            return;
        }

        search_reviews.dispatch(query_text);
    };

    let render_stars = move |rating: u8| {
        (1..=5)
            .map(|i| if i <= rating { "⭐" } else { "☆" })
            .collect::<String>()
    };

    view! {
        <div class="search-interface">
            <form on:submit=on_search class="search-form">
                <div class="search-input-group">
                    <input
                        type="text"
                        placeholder="Search reviews using natural language..."
                        prop:value=query
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                        disabled=is_searching
                        class="search-input"
                    />
                    <button
                        type="submit"
                        disabled=is_searching
                        class="search-btn"
                    >
                        {move || if is_searching.get() { "Searching..." } else { "🔍 Search" }}
                    </button>
                </div>
            </form>

            <div class="search-examples">
                <p><strong>"Try searching for:"</strong></p>
                <div class="example-queries">
                    <button
                        class="example-btn"
                        on:click=move |_| set_query.set("great quality product".to_string())
                    >
                        "great quality product"
                    </button>
                    <button
                        class="example-btn"
                        on:click=move |_| set_query.set("fast delivery".to_string())
                    >
                        "fast delivery"
                    </button>
                    <button
                        class="example-btn"
                        on:click=move |_| set_query.set("value for money".to_string())
                    >
                        "value for money"
                    </button>
                </div>
            </div>

            {move || {
                if !error.get().is_empty() {
                    view! { <div class="error-message">{error.get()}</div> }.into_view()
                } else if no_results.get() {
                    view! {
                        <div class="no-results">
                            <p>"No similar reviews found for your query."</p>
                            <p>"Try using different keywords or phrases."</p>
                        </div>
                    }.into_view()
                } else if !search_results.get().is_empty() {
                    view! {
                        <div class="search-results">
                            <h3>{format!("Found {} similar reviews:", search_results.get().len())}</h3>
                            <div class="results-list">
                                {search_results.get().into_iter().map(|result| {
                                    view! {
                                        <div class="result-item">
                                            <div class="result-header">
                                                <h4 class="result-title">{result.review.title}</h4>
                                                <div class="result-meta">
                                                    <span class="similarity-score">
                                                        {format!("Similarity: {:.1}%", result.similarity_score * 100.0)}
                                                    </span>
                                                    <span class="rating">
                                                        {render_stars(result.review.rating)}
                                                    </span>
                                                </div>
                                            </div>
                                            <p class="result-body">{result.review.body}</p>
                                            <div class="result-footer">
                                                <span class="product-id">
                                                    "Product: " {result.review.product_id}
                                                </span>
                                                <span class="timestamp">
                                                    {result.review.timestamp}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}
