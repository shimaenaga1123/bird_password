use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[server(GeneratePassword, "/api")]
pub async fn generate_password() -> Result<String, ServerFnError> {
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    let txt_path = "bird_names.txt";

    let file = File::open(txt_path)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let reader = BufReader::new(file);
    let words: Vec<String> = reader
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

    let mut rng = rand::thread_rng();
    let selected_words: Vec<&String> = words.choose_multiple(&mut rng, 4).collect();

    let mut parts = Vec::new();
    for word in selected_words {
        let digit = rng.gen_range(0..10);
        let part = format!("{}{}", word, digit);
        parts.push(part);
    }

    Ok(parts.join("-"))
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="style.css"/>
        <Title text="Bird Password Generator"/>
        <Router>
            <Routes>
                <Route path="" view=HomePage/>
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let generate_action = create_server_action::<GeneratePassword>();
    let value = generate_action.value();

    view! {
        <div class="container">
            <h1>Bird Password Generator</h1>
            <ActionForm action=generate_action>
                <button type="submit">"Generate Password"</button>
            </ActionForm>
            <div class="result">
                {move || match value.get() {
                    Some(Ok(password)) => view! { <p class="password">{password}</p> }.into_view(),
                    Some(Err(e)) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_view(),
                    None => view!{<span></span>}.into_view(),
                }}
            </div>
        </div>
    }
}

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            console_error_panic_hook::set_once();
            leptos::mount_to_body(App);
        }
    }
}
