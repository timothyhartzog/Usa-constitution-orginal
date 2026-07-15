use leptos::prelude::*;
use crate::api::SearchResult;
use crate::components::result_card::ResultCard;
use std::sync::Arc;

#[component]
pub fn ResultsList(
    results: Vec<SearchResult>,
    count: usize,
    search_type: String,
) -> impl IntoView {
    let results = Arc::new(results);
    
    // Pagination state
    let (page, set_page) = signal(1);
    let items_per_page = 10;
    
    let total_pages = {
        let results = results.clone();
        Memo::new(move |_| {
            let len = results.len();
            if len == 0 { 1 } else { (len as f64 / items_per_page as f64).ceil() as usize }
        })
    };

    let paginated_results = {
        let results = results.clone();
        move || {
            let p = page.get();
            let start = (p - 1) * items_per_page;
            let end = (start + items_per_page).min(results.len());
            if start < results.len() {
                results[start..end].to_vec()
            } else {
                vec![]
            }
        }
    };

    if count == 0 {
        return view! { <div class="text-center text-slate-500 mt-8">"No results found"</div> }.into_any();
    }

    view! {
        <div class="mt-8">
            <div class="text-sm text-slate-500 mb-4 flex justify-between items-center">
                <span>{format!("Found {} results using {} search", count, search_type)}</span>
                
                // Pagination controls top
                <div class="flex space-x-2">
                    <button 
                        class="px-2 py-1 bg-slate-200 rounded disabled:opacity-50"
                        disabled=move || page.get() == 1
                        on:click=move |_| set_page.update(|p| *p -= 1)
                    >"Prev"</button>
                    <span class="px-2 py-1">"Page " {move || page.get()} " of " {move || total_pages.get()}</span>
                    <button 
                        class="px-2 py-1 bg-slate-200 rounded disabled:opacity-50"
                        disabled=move || page.get() == total_pages.get()
                        on:click=move |_| set_page.update(|p| *p += 1)
                    >"Next"</button>
                </div>
            </div>
            
            <div class="space-y-4">
                {move || paginated_results().into_iter().map(|result| {
                    view! { <ResultCard result=result /> }
                }).collect::<Vec<_>>()}
            </div>
            
            // Pagination controls bottom
            <div class="flex justify-center space-x-2 mt-8">
                <button 
                    class="px-4 py-2 bg-slate-200 rounded disabled:opacity-50 hover:bg-slate-300 transition-colors"
                    disabled=move || page.get() == 1
                    on:click=move |_| set_page.update(|p| *p -= 1)
                >"Previous Page"</button>
                <button 
                    class="px-4 py-2 bg-slate-200 rounded disabled:opacity-50 hover:bg-slate-300 transition-colors"
                    disabled=move || page.get() == total_pages.get()
                    on:click=move |_| set_page.update(|p| *p += 1)
                >"Next Page"</button>
            </div>
        </div>
    }.into_any()
}
