use bm25::{Document, Language, SearchEngine, SearchEngineBuilder};
use gloo_net::http::Request;
use leptos::prelude::*;

use csv::Reader;
use web_sys::console;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub title: String,
    pub recipe: String,
}

impl From<Recipe> for Document<String> {
    fn from(value: Recipe) -> Self {
        Document::new(value.title, value.recipe)
    }
}

async fn load_data() -> Vec<Recipe> {
    let response = Request::get("/data/recipes_en.csv")
        .send()
        .await
        .expect("recipes_en.csv");
    let text = response.text().await.expect("text");
    let mut reader = Reader::from_reader(text.as_bytes());
    reader
        .records()
        .map(|r| r.unwrap())
        .map(|r| {
            let title = r.get(0).expect("title").to_string();
            let recipe = r.get(1).expect("recipe").to_string();
            Recipe { title, recipe }
        })
        .collect()
}

async fn get_search_engine() -> SearchEngine<String, u32> {
    let recipes = load_data().await;
    SearchEngineBuilder::with_documents(Language::English, recipes).build()
}

#[component]
fn App() -> impl IntoView {
    let search_engine = LocalResource::new(move || get_search_engine());
    // we can access the resource values with .get()
    // this will reactively return None before the Future has resolved
    // and update to Some(T) when it has resolved
    let search = move |query: &str| {
        search_engine
            .read()
            .as_deref()
            .map(|value| {
                let search_engine = value;
                let results = search_engine.search(query, 5);
                let results = results
                    .iter()
                    .map(|result| result.document.contents.clone())
                    .collect::<Vec<_>>();
                format!("{:?}", results)
            })
            // This loading state will only show before the first load
            .unwrap_or_else(|| "Loading...".into())
    };

    let (result, set_result) = signal("None".to_string());

    view! {
        <input type="text"
            // adding :target gives us typed access to the element
            // that is the target of the event that fires
            on:input:target=move |ev| {
                set_result.set(search(&ev.target().value()));
            }

            //// the `prop:` syntax lets you update a DOM property,
            //// rather than an attribute.
            //prop:value=
        />
        <p>"Result is: " {result}</p>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
