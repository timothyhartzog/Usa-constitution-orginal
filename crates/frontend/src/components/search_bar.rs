use leptos::prelude::*;
use leptos::ev::Event;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;

#[component]
pub fn SearchBar(
    query: ReadSignal<String>,
    set_query: WriteSignal<String>,
    search_type: ReadSignal<String>,
    set_search_type: WriteSignal<String>,
) -> impl IntoView {
    // We use a local state for the input value to allow typing smoothly
    let (input_val, set_input_val) = signal(query.get());
    
    // Timer for debouncing
    let timer = Rc::new(RefCell::new(None::<Timeout>));

    let on_input = move |ev: Event| {
        let val = event_target_value(&ev);
        set_input_val.set(val.clone());
        
        let mut timer_ref = timer.borrow_mut();
        if let Some(t) = timer_ref.take() {
            t.cancel();
        }
        
        *timer_ref = Some(Timeout::new(300, move || {
            set_query.set(val);
        }));
    };

    view! {
        <div class="flex flex-col items-center justify-center w-full max-w-2xl mx-auto mt-8">
            <div class="relative w-full">
                <input
                    type="text"
                    prop:value=input_val
                    on:input=on_input
                    class="w-full px-4 py-3 text-lg border-2 border-slate-300 rounded-lg focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary shadow-sm"
                    placeholder="Search the Constitution (e.g., 'freedom of speech', 'taxes')"
                />
                <button
                    class="absolute right-2 top-2 bottom-2 px-4 bg-primary text-white font-semibold rounded-md hover:bg-blue-700 transition-colors"
                >
                    "Search"
                </button>
            </div>
            
            <div class="flex flex-row space-x-4 mt-4 text-sm text-slate-600">
                <label class="flex items-center space-x-1 cursor-pointer">
                    <input 
                        type="radio" 
                        name="search_type" 
                        value="fulltext" 
                        checked=move || search_type.get() == "fulltext"
                        on:change=move |_| set_search_type.set("fulltext".to_string())
                        class="text-primary focus:ring-primary" 
                    />
                    <span>"Exact/Full-text"</span>
                </label>
                <label class="flex items-center space-x-1 cursor-pointer">
                    <input 
                        type="radio" 
                        name="search_type" 
                        value="fuzzy" 
                        checked=move || search_type.get() == "fuzzy"
                        on:change=move |_| set_search_type.set("fuzzy".to_string())
                        class="text-primary focus:ring-primary" 
                    />
                    <span>"Fuzzy"</span>
                </label>
                <label class="flex items-center space-x-1 cursor-pointer">
                    <input 
                        type="radio" 
                        name="search_type" 
                        value="semantic" 
                        checked=move || search_type.get() == "semantic"
                        on:change=move |_| set_search_type.set("semantic".to_string())
                        class="text-primary focus:ring-primary" 
                    />
                    <span>"Semantic/Concept"</span>
                </label>
            </div>
        </div>
    }
}
