use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

#[component]
pub fn BulkUpload() -> impl IntoView {
    let (selected_file, set_selected_file) = create_signal(Option::<String>::None);
    let (is_uploading, set_is_uploading) = create_signal(false);
    let (upload_result, set_upload_result) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());

    let file_input_ref = create_node_ref::<leptos::html::Input>();

    let on_file_change = move |ev: Event| {
        let target = ev.target().unwrap();
        let input: HtmlInputElement = target.dyn_into().unwrap();
        
        if let Some(files) = input.files() {
            if files.length() > 0 {
                if let Some(file) = files.get(0) {
                    set_selected_file.set(Some(file.name()));
                    set_error.set(String::new());
                    set_upload_result.set(String::new());
                }
            }
        }
    };

    let upload_file = create_action(move |_: &()| {
        async move {
            set_is_uploading.set(true);
            set_error.set(String::new());
            set_upload_result.set(String::new());
            
            // TODO: Replace with actual file upload logic
            // For now, just simulate a successful upload
            gloo_timers::future::TimeoutFuture::new(2000).await;
            
            set_is_uploading.set(false);
            set_upload_result.set("File uploaded successfully! (Placeholder - 0 reviews processed)".to_string());
            set_selected_file.set(None);
            
            // Clear file input
            if let Some(input) = file_input_ref.get() {
                input.set_value("");
            }
        }
    });

    let on_upload = move |_| {
        if selected_file.get().is_none() {
            set_error.set("Please select a file first".to_string());
            return;
        }
        
        upload_file.dispatch(());
    };

    view! {
        <div class="bulk-upload">
            <div class="upload-section">
                <div class="form-group">
                    <label for="file-input">"Select File:"</label>
                    <input
                        type="file"
                        id="file-input"
                        accept=".json,.jsonl,.csv"
                        on:change=on_file_change
                        disabled=is_uploading
                        node_ref=file_input_ref
                    />
                </div>
                
                {move || {
                    if let Some(filename) = selected_file.get() {
                        view! {
                            <div class="selected-file">
                                <span>"Selected: "</span>
                                <strong>{filename}</strong>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                
                <button 
                    on:click=on_upload
                    disabled=move || is_uploading.get() || selected_file.get().is_none()
                    class="upload-btn"
                >
                    {move || if is_uploading.get() { "Uploading..." } else { "Upload File" }}
                </button>
            </div>
            
            <div class="file-format-info">
                <h4>"Supported Formats:"</h4>
                <ul>
                    <li>"JSON - Array of review objects"</li>
                    <li>"JSONL - One review object per line"</li>
                    <li>"CSV - With title, body, product_id, rating columns"</li>
                </ul>
                <p class="format-example">
                    <strong>"Example JSON format:"</strong><br/>
                    <code>
                        r#"[{"title": "Great product", "body": "Really loved it", "product_id": "prod123", "rating": 5}]"#
                    </code>
                </p>
            </div>
            
            {move || {
                if !error.get().is_empty() {
                    view! { <div class="error-message">{error.get()}</div> }.into_view()
                } else if !upload_result.get().is_empty() {
                    view! { <div class="success-message">{upload_result.get()}</div> }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}