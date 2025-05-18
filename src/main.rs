use bm25::{Document, Language, SearchEngine, SearchEngineBuilder, SearchResult};
use csv::Reader;
use gloo_net::http::Request;
use leptos::{either::Either, prelude::*};

pub struct Recipe {
    pub title: String,
    pub method: String,
}

impl From<Recipe> for Document<String> {
    fn from(value: Recipe) -> Self {
        Document::new(value.title, value.method)
    }
}

async fn load_recipes() -> Vec<Recipe> {
    let response = Request::get("recipes.csv")
        .send()
        .await
        .expect("recipes to be available");
    let text = response.text().await.expect("recipes to be text");
    let mut reader = Reader::from_reader(text.as_bytes());
    reader
        .records()
        .filter_map(|r| r.ok())
        .map(|r| {
            let title = r.get(0).expect("recipe title").to_string();
            let method = r.get(1).expect("recipe method").to_string();
            Recipe { title, method }
        })
        .collect()
}

async fn get_search_engine() -> SearchEngine<String, u32> {
    let recipes = load_recipes().await;
    SearchEngineBuilder::with_documents(Language::English, recipes).build()
}

#[component]
pub fn App() -> impl IntoView {
    let (search, set_search) = signal((String::new(), Vec::<SearchResult<String>>::new()));

    let search_engine = LocalResource::new(get_search_engine);

    let on_query_input = move |query: String| {
        if let Some(se) = search_engine.read().as_ref() {
            let results = se.search(&query, 5);
            set_search.set((query, results));
        }
    };

    view! {
        <div class="inner-body">
            <header>
                <h1>"bm25 demo"</h1>
                <p>
                    "This is a demo of the keyword search engine from the "
                    <a href="https://github.com/Michael-JB/bm25">"bm25"</a>" Rust crate.
                    Use the search bar below to query a small collection of recipes.
                    For example, try searching for 'cinnamon' or 'noodle'. Only the top 5 results
                    are shown. The recipe titles are not included in the search. These recipes are
                    AI-generated; I do not recommend trying them at home..."
                </p>
                <p>
                    "This demo is written in Rust and compiles to WebAssembly to run directly
                    in the browser. See the "
                    <a href="https://github.com/Michael-JB/bm25-demo">"source code"</a>"."
                </p>
            </header>
            <input
                class="search-input"
                type="search"
                placeholder="Search for a recipe..."
                on:input:target=move |ev| {
                    on_query_input(ev.target().value());
                }
            />

            {move || {
                let (ref query, ref results) = *search.read();

                if query.is_empty() {
                    return None;
                }

                let table_head = view! {
                    <tr>
                        <th class="score-header">"Score"</th>
                        <th>"Recipe"</th>
                    </tr>
                };
                let table_body = if results.is_empty() {
                    Either::Left(view! {<tr><td colspan="2">"No results."</td></tr>})
                } else {
                    Either::Right(results.iter().map(|result| {
                        view! {
                            <tr>
                                <td>{format!("{:.3}", result.score)}</td>
                                <td>
                                    <h1 class="recipe-title">{result.document.id.clone()}</h1>
                                    <p class="recipe-contents">{result.document.contents.clone()}</p>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>())
                };
                Some(view! {
                    <table class="results-table">
                        <thead>{table_head}</thead>
                        <tbody>{table_body}</tbody>
                    </table>
                })
            }}
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
