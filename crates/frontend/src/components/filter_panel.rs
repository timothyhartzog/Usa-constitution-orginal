use leptos::prelude::*;
use crate::api::SearchFilters;

#[component]
pub fn FilterPanel(
    filters: ReadSignal<SearchFilters>,
    set_filters: WriteSignal<SearchFilters>,
) -> impl IntoView {
    let on_doc_change = move |ev| {
        let val = event_target_value(&ev);
        let mut f = filters.get();
        if val.is_empty() {
            f.document_id = None;
        } else {
            f.document_id = Some(val);
        }
        set_filters.set(f);
    };

    let on_author_change = move |ev| {
        let val = event_target_value(&ev);
        let mut f = filters.get();
        if val.is_empty() {
            f.author = None;
        } else {
            f.author = Some(val);
        }
        set_filters.set(f);
    };

    let on_issue_tag_change = move |ev| {
        let val = event_target_value(&ev);
        let mut f = filters.get();
        if val.is_empty() {
            f.issue_tag = None;
        } else {
            f.issue_tag = Some(val);
        }
        set_filters.set(f);
    };

    let on_clause_tag_change = move |ev| {
        let val = event_target_value(&ev);
        let mut f = filters.get();
        if val.is_empty() {
            f.clause_tag = None;
        } else {
            f.clause_tag = Some(val);
        }
        set_filters.set(f);
    };

    view! {
        <div class="w-full bg-white border border-slate-200 rounded-lg shadow-sm p-4 mt-4">
            <h3 class="font-semibold text-slate-800 mb-3">"Filters"</h3>
            <div class="grid grid-cols-1 sm:grid-cols-4 gap-4">
                // Document Filter
                <div class="flex flex-col space-y-1">
                    <label class="text-xs font-medium text-slate-500">"Document"</label>
                    <select
                        on:change=on_doc_change
                        class="px-3 py-2 border border-slate-300 rounded text-sm focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary"
                    >
                        <option value="">"All Documents"</option>
                        <option value="constitution">"US Constitution"</option>
                        <option value="federalist">"Federalist Papers"</option>
                        <option value="bill_of_rights">"Bill of Rights"</option>
                        <option value="amendments">"Other Amendments"</option>
                    </select>
                </div>
                
                // Author Filter
                <div class="flex flex-col space-y-1">
                    <label class="text-xs font-medium text-slate-500">"Author"</label>
                    <select
                        on:change=on_author_change
                        class="px-3 py-2 border border-slate-300 rounded text-sm focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary"
                    >
                        <option value="">"Any Author"</option>
                        <option value="Alexander Hamilton">"Alexander Hamilton"</option>
                        <option value="James Madison">"James Madison"</option>
                        <option value="John Jay">"John Jay"</option>
                        <option value="Gouverneur Morris">"Gouverneur Morris"</option>
                    </select>
                </div>

                // Issue Tag Filter
                <div class="flex flex-col space-y-1">
                    <label class="text-xs font-medium text-slate-500">"Issue Topic"</label>
                    <select
                        on:change=on_issue_tag_change
                        class="px-3 py-2 border border-slate-300 rounded text-sm focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary"
                    >
                        <option value="">"Any Topic"</option>
                        <option value="taxes">"Taxes"</option>
                        <option value="liberty">"Liberty"</option>
                        <option value="rights">"Rights"</option>
                        <option value="powers">"Powers"</option>
                        <option value="commerce">"Commerce"</option>
                    </select>
                </div>
                
                // Clause Tag Filter
                <div class="flex flex-col space-y-1">
                    <label class="text-xs font-medium text-slate-500">"Clause Tag"</label>
                    <select
                        on:change=on_clause_tag_change
                        class="px-3 py-2 border border-slate-300 rounded text-sm focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary"
                    >
                        <option value="">"Any Clause"</option>
                        <option value="commerce_clause">"Commerce Clause"</option>
                        <option value="necessary_and_proper">"Necessary and Proper"</option>
                        <option value="supremacy_clause">"Supremacy Clause"</option>
                        <option value="equal_protection">"Equal Protection"</option>
                        <option value="due_process">"Due Process"</option>
                    </select>
                </div>
            </div>
        </div>
    }
}
