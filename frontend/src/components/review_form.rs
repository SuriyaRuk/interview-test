use leptos::*;
use serde::{Deserialize, Serialize};
use leptos::ev::SubmitEvent;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewData {
    pub title: String,
    pub body: String,
    pub product_id: String,
    pub rating: u8,
}

#[component]
pub fn ReviewForm() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (body, set_body) = create_signal(String::new());
    let (product_id, set_product_id) = create_signal(String::new());
    let (rating, set_rating) = create_signal(5u8);
    let (is_loading, set_is_loading) = create_signal(false);
    let (message, set_message) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let submit_review = create_action(move |_: &()| {
        let _review_data = ReviewData {
            title: title.get(),
            body: body.get(),
            product_id: product_id.get(),
            rating: rating.get(),
        };
        
        async move {
            set_is_loading.set(true);
            set_error.set(String::new());
            set_message.set(String::new());
            
            // TODO: Replace with actual API call
            // For now, just simulate a successful submission
            gloo_timers::future::TimeoutFuture::new(1000).await;
            
            set_is_loading.set(false);
            set_message.set("Review submitted successfully! (Placeholder)".to_string());
            
            // Clear form
            set_title.set(String::new());
            set_body.set(String::new());
            set_product_id.set(String::new());
            set_rating.set(5);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        
        // Basic validation
        if title.get().trim().is_empty() {
            set_error.set("Title is required".to_string());
            return;
        }
        if body.get().trim().is_empty() {
            set_error.set("Review body is required".to_string());
            return;
        }
        if product_id.get().trim().is_empty() {
            set_error.set("Product ID is required".to_string());
            return;
        }
        
        submit_review.dispatch(());
    };

    view! {
        <div class="review-form">
            <form on:submit=on_submit>
                <div class="form-group">
                    <label for="title">"Review Title:"</label>
                    <input
                        type="text"
                        id="title"
                        placeholder="Enter review title"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        disabled=is_loading
                    />
                </div>
                
                <div class="form-group">
                    <label for="body">"Review:"</label>
                    <textarea
                        id="body"
                        placeholder="Write your review here..."
                        rows="4"
                        prop:value=body
                        on:input=move |ev| set_body.set(event_target_value(&ev))
                        disabled=is_loading
                    ></textarea>
                </div>
                
                <div class="form-group">
                    <label for="product_id">"Product ID:"</label>
                    <input
                        type="text"
                        id="product_id"
                        placeholder="Enter product ID"
                        prop:value=product_id
                        on:input=move |ev| set_product_id.set(event_target_value(&ev))
                        disabled=is_loading
                    />
                </div>
                
                <div class="form-group">
                    <label for="rating">"Rating:"</label>
                    <select
                        id="rating"
                        prop:value=move || rating.get().to_string()
                        on:change=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                set_rating.set(val);
                            }
                        }
                        disabled=is_loading
                    >
                        <option value="1">"1 Star"</option>
                        <option value="2">"2 Stars"</option>
                        <option value="3">"3 Stars"</option>
                        <option value="4">"4 Stars"</option>
                        <option value="5" selected>"5 Stars"</option>
                    </select>
                </div>
                
                <button type="submit" disabled=is_loading class="submit-btn">
                    {move || if is_loading.get() { "Submitting..." } else { "Submit Review" }}
                </button>
            </form>
            
            {move || {
                if !error.get().is_empty() {
                    view! { <div class="error-message">{error.get()}</div> }.into_view()
                } else if !message.get().is_empty() {
                    view! { <div class="success-message">{message.get()}</div> }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}