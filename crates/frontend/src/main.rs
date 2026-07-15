use leptos::prelude::*;
use leptos::mount::mount_to_body;

mod components;
mod api;
use components::search_bar::SearchBar;
use components::filter_panel::FilterPanel;
use components::results_list::ResultsList;
use api::{SearchRequest, perform_search, SearchFilters};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> })
}

#[component]
fn App() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (search_type, set_search_type) = signal("fulltext".to_string());
    let (filters, set_filters) = signal(SearchFilters::default());
    
    // Resource to fetch search results when query, search_type, or filters changes
    let search_results = LocalResource::new(move || {
        let q = query.get();
        let st = search_type.get();
        let f = filters.get();
        async move {
            if q.trim().is_empty() {
                return Ok(api::SearchResponse {
                    results: vec![],
                    count: 0,
                    search_type: st,
                });
            }
            let req = SearchRequest {
                query: q,
                search_type: Some(st),
                max_results: Some(50),
                filters: Some(f),
            };
            perform_search(&req).await
        }
    });

    view! {
        <div class="container mx-auto p-4">
            <h1 class="text-4xl font-bold text-center text-slate-800">
                "Constitutional Research System"
            </h1>
            <p class="text-center text-slate-600 mt-2">
                "Search the U.S. Constitution and Founding Documents"
            </p>
            
            <SearchBar 
                query=query 
                set_query=set_query 
                search_type=search_type 
                set_search_type=set_search_type 
            />
            
            <div class="max-w-2xl mx-auto">
                <FilterPanel filters=filters set_filters=set_filters />
            </div>
            
            <div class="mt-8 max-w-4xl mx-auto">
                <Suspense fallback=move || view! { <div class="text-center">"Loading..."</div> }>
                    {move || {
                        search_results.get().map(|res| match res {
                            Ok(response) => {
                                view! {
                                    <ResultsList 
                                        results=response.results 
                                        count=response.count 
                                        search_type=response.search_type 
                                    />
                                }.into_any()
                            },
                            Err(e) => view! { <div class="text-red-500">"Error: " {e}</div> }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
