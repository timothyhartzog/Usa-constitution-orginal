use leptos::prelude::*;
use crate::api::SearchResult;

#[component]
pub fn ResultCard(result: SearchResult) -> impl IntoView {
    view! {
        <div class="p-5 bg-white border border-slate-200 rounded-lg shadow-sm hover:shadow-md transition-shadow cursor-pointer">
            <div class="flex justify-between items-start">
                <h3 class="font-bold text-lg text-primary">{result.document_title}</h3>
                <span class="text-xs bg-slate-100 text-slate-600 px-2 py-1 rounded border border-slate-200">
                    "Score: " {format!("{:.2}", result.score)}
                </span>
            </div>
            
            {result.chunk_title.map(|t| view! { <h4 class="font-medium text-slate-700 mt-1">{t}</h4> })}
            
            <div class="flex space-x-4 mt-2 text-xs text-slate-500">
                {result.document_author.map(|a| view! { <span><i class="opacity-70 mr-1">"By"</i> {a}</span> })}
            </div>
            
            <p class="mt-3 text-slate-800 leading-relaxed">
                {result.preview}
            </p>
        </div>
    }
}
