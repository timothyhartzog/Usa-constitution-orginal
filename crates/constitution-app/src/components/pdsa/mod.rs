use dioxus::prelude::*;

use crate::state::{use_pdsa, PdsaCycle, PdsaDraft, PdsaStage, PdsaStatus};

#[component]
pub fn PlanDoStudyActPage() -> Element {
    let mut pdsa = use_pdsa();
    let state_snapshot = pdsa.read().clone();
    let cycles = state_snapshot.cycles.clone();
    let selected_cycle = state_snapshot.selected_cycle().cloned();
    let active_count = cycles
        .iter()
        .filter(|cycle| cycle.status == PdsaStatus::Active)
        .count();
    let complete_count = cycles
        .iter()
        .filter(|cycle| cycle.status == PdsaStatus::Complete)
        .count();

    let save_draft = move |draft: PdsaDraft| {
        pdsa.write().draft = draft;
        crate::persist_pdsa_state(&pdsa.read());
    };

    rsx! {
        div { class: "page pdsa-page",
            header { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h2 { "Plan Do Study Act" }
                        p { class: "page-subtitle",
                            "Manage improvement cycles from aim statement through learning and next action."
                        }
                    }
                    div { class: "pdsa-summary",
                        SummaryStat { label: "Cycles".to_string(), value: cycles.len().to_string() }
                        SummaryStat { label: "Active".to_string(), value: active_count.to_string() }
                        SummaryStat { label: "Complete".to_string(), value: complete_count.to_string() }
                    }
                }
            }

            section { class: "pdsa-layout",
                aside { class: "pdsa-list-panel",
                    div { class: "pdsa-panel-header",
                        h3 { "Cycles" }
                    }
                    if cycles.is_empty() {
                        div { class: "empty-state pdsa-empty",
                            h3 { "No cycles yet" }
                            p { "Create a cycle to start tracking a change idea." }
                        }
                    } else {
                        div { class: "pdsa-cycle-list",
                            for cycle in cycles.iter() {
                                {
                                    let id = cycle.id.clone();
                                    let is_selected = selected_cycle
                                        .as_ref()
                                        .map(|selected| selected.id == cycle.id)
                                        .unwrap_or(false);
                                    rsx! {
                                        button {
                                            key: "{cycle.id}",
                                            class: if is_selected { "pdsa-cycle-item pdsa-cycle-item-active" } else { "pdsa-cycle-item" },
                                            onclick: move |_| {
                                                pdsa.write().selected_id = Some(id.clone());
                                                crate::persist_pdsa_state(&pdsa.read());
                                            },
                                            span { class: "pdsa-cycle-title", "{cycle.title}" }
                                            span { class: "pdsa-cycle-meta",
                                                "{cycle.stage.label()} · {cycle.status.label()}"
                                            }
                                            div { class: "pdsa-progress-track",
                                                div {
                                                    class: "pdsa-progress-fill",
                                                    style: "width: {cycle.stage.progress_percent()}%;",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "pdsa-main-panel",
                    NewCycleForm { draft: state_snapshot.draft.clone(), onsave: save_draft }
                    if let Some(cycle) = selected_cycle {
                        CycleEditor { cycle }
                    } else {
                        div { class: "pdsa-detail-empty",
                            h3 { "Select or create a cycle" }
                            p { "The working record will appear here." }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SummaryStat(label: String, value: String) -> Element {
    rsx! {
        div { class: "pdsa-summary-stat",
            span { class: "pdsa-summary-value", "{value}" }
            span { class: "pdsa-summary-label", "{label}" }
        }
    }
}

#[component]
fn NewCycleForm(draft: PdsaDraft, onsave: EventHandler<PdsaDraft>) -> Element {
    let mut pdsa = use_pdsa();
    let mut title = use_signal(|| draft.title);
    let mut aim = use_signal(|| draft.aim);
    let mut owner = use_signal(|| draft.owner);
    let mut metric = use_signal(|| draft.metric);
    let mut baseline = use_signal(|| draft.baseline);
    let mut target = use_signal(|| draft.target);
    let mut plan = use_signal(|| draft.plan);
    let mut prediction = use_signal(|| draft.prediction);
    let mut message = use_signal(|| Option::<String>::None);

    let persist_current_draft = move || {
        onsave.call(PdsaDraft {
            title: title.read().clone(),
            aim: aim.read().clone(),
            owner: owner.read().clone(),
            metric: metric.read().clone(),
            baseline: baseline.read().clone(),
            target: target.read().clone(),
            plan: plan.read().clone(),
            prediction: prediction.read().clone(),
        });
    };

    rsx! {
        section { class: "pdsa-create-panel",
            div { class: "pdsa-panel-header",
                h3 { "New cycle" }
                if let Some(text) = message.read().clone() {
                    span { class: "editor-status", "{text}" }
                }
            }
            div { class: "pdsa-form-grid",
                label { class: "pdsa-field pdsa-field-wide",
                    span { "Title" }
                    input {
                        r#type: "text",
                        value: "{title}",
                        placeholder: "Example: Improve source tagging coverage",
                        oninput: move |e| {
                            title.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field pdsa-field-wide",
                    span { "Aim" }
                    textarea {
                        value: "{aim}",
                        placeholder: "What should improve, for whom, by how much, and by when?",
                        oninput: move |e| {
                            aim.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field",
                    span { "Owner" }
                    input {
                        r#type: "text",
                        value: "{owner}",
                        oninput: move |e| {
                            owner.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field",
                    span { "Measure" }
                    input {
                        r#type: "text",
                        value: "{metric}",
                        placeholder: "Primary metric",
                        oninput: move |e| {
                            metric.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field",
                    span { "Baseline" }
                    input {
                        r#type: "text",
                        value: "{baseline}",
                        oninput: move |e| {
                            baseline.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field",
                    span { "Target" }
                    input {
                        r#type: "text",
                        value: "{target}",
                        oninput: move |e| {
                            target.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field pdsa-field-wide",
                    span { "Plan" }
                    textarea {
                        value: "{plan}",
                        placeholder: "What change will be tested?",
                        oninput: move |e| {
                            plan.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
                label { class: "pdsa-field pdsa-field-wide",
                    span { "Prediction" }
                    textarea {
                        value: "{prediction}",
                        placeholder: "What do you expect will happen?",
                        oninput: move |e| {
                            prediction.set(e.value());
                            persist_current_draft();
                        },
                    }
                }
            }
            div { class: "pdsa-actions",
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| {
                        title.set(String::new());
                        aim.set(String::new());
                        owner.set(String::new());
                        metric.set(String::new());
                        baseline.set(String::new());
                        target.set(String::new());
                        plan.set(String::new());
                        prediction.set(String::new());
                        pdsa.write().draft = PdsaDraft::default();
                        crate::persist_pdsa_state(&pdsa.read());
                        message.set(Some("Draft cleared".to_string()));
                    },
                    "Clear"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let title_value = title.read().trim().to_string();
                        if title_value.is_empty() {
                            message.set(Some("Title is required".to_string()));
                            return;
                        }
                        let today = crate::today_iso();
                        let id = next_cycle_id(&title_value, pdsa.read().cycles.len(), &today);
                        let cycle = PdsaCycle {
                            id: id.clone(),
                            title: title_value,
                            aim: aim.read().clone(),
                            owner: owner.read().clone(),
                            metric: metric.read().clone(),
                            baseline: baseline.read().clone(),
                            target: target.read().clone(),
                            plan: plan.read().clone(),
                            prediction: prediction.read().clone(),
                            doing: String::new(),
                            study: String::new(),
                            act: String::new(),
                            stage: PdsaStage::Plan,
                            status: PdsaStatus::Active,
                            created_at: today.clone(),
                            updated_at: today,
                        };
                        {
                            let mut state = pdsa.write();
                            state.cycles.insert(0, cycle);
                            state.selected_id = Some(id);
                            state.draft = PdsaDraft::default();
                        }
                        crate::persist_pdsa_state(&pdsa.read());
                        title.set(String::new());
                        aim.set(String::new());
                        owner.set(String::new());
                        metric.set(String::new());
                        baseline.set(String::new());
                        target.set(String::new());
                        plan.set(String::new());
                        prediction.set(String::new());
                        message.set(Some("Cycle created".to_string()));
                    },
                    "Create cycle"
                }
            }
        }
    }
}

#[component]
fn CycleEditor(cycle: PdsaCycle) -> Element {
    let mut pdsa = use_pdsa();
    let cycle_id = cycle.id.clone();

    rsx! {
        section { class: "pdsa-detail-panel",
            div { class: "pdsa-detail-header",
                div {
                    h3 { "{cycle.title}" }
                    p { class: "pdsa-detail-meta",
                        "Created {cycle.created_at}"
                        if !cycle.owner.trim().is_empty() {
                            " · Owner: {cycle.owner}"
                        }
                    }
                }
                div { class: "pdsa-detail-controls",
                    select {
                        value: "{cycle.stage.as_str()}",
                        onchange: {
                            let id = cycle_id.clone();
                            move |e| update_cycle(&mut pdsa, &id, |cycle| {
                                cycle.stage = PdsaStage::from_str(&e.value());
                            })
                        },
                        for stage in PdsaStage::ALL {
                            option { value: "{stage.as_str()}", "{stage.label()}" }
                        }
                    }
                    select {
                        value: "{cycle.status.as_str()}",
                        onchange: {
                            let id = cycle_id.clone();
                            move |e| update_cycle(&mut pdsa, &id, |cycle| {
                                cycle.status = PdsaStatus::from_str(&e.value());
                            })
                        },
                        for status in PdsaStatus::ALL {
                            option { value: "{status.as_str()}", "{status.label()}" }
                        }
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: {
                            let id = cycle_id.clone();
                            move |_| {
                                {
                                    let mut state = pdsa.write();
                                    state.cycles.retain(|cycle| cycle.id != id);
                                    state.selected_id = state.cycles.first().map(|cycle| cycle.id.clone());
                                }
                                crate::persist_pdsa_state(&pdsa.read());
                            }
                        },
                        "Delete"
                    }
                }
            }
            div { class: "pdsa-stage-bar",
                for stage in PdsaStage::ALL {
                    div {
                        class: if stage.progress_percent() <= cycle.stage.progress_percent() {
                            "pdsa-stage-step pdsa-stage-step-done"
                        } else {
                            "pdsa-stage-step"
                        },
                        "{stage.label()}"
                    }
                }
            }
            div { class: "pdsa-measure-grid",
                MeasureBlock { label: "Aim".to_string(), value: cycle.aim.clone() }
                MeasureBlock { label: "Measure".to_string(), value: cycle.metric.clone() }
                MeasureBlock { label: "Baseline".to_string(), value: cycle.baseline.clone() }
                MeasureBlock { label: "Target".to_string(), value: cycle.target.clone() }
            }
            div { class: "pdsa-notes-grid",
                CycleTextArea {
                    id: cycle_id.clone(),
                    label: "Plan".to_string(),
                    value: cycle.plan.clone(),
                    field: PdsaTextField::Plan,
                }
                CycleTextArea {
                    id: cycle_id.clone(),
                    label: "Prediction".to_string(),
                    value: cycle.prediction.clone(),
                    field: PdsaTextField::Prediction,
                }
                CycleTextArea {
                    id: cycle_id.clone(),
                    label: "Do".to_string(),
                    value: cycle.doing.clone(),
                    field: PdsaTextField::Do,
                }
                CycleTextArea {
                    id: cycle_id.clone(),
                    label: "Study".to_string(),
                    value: cycle.study.clone(),
                    field: PdsaTextField::Study,
                }
                CycleTextArea {
                    id: cycle_id.clone(),
                    label: "Act".to_string(),
                    value: cycle.act,
                    field: PdsaTextField::Act,
                }
            }
        }
    }
}

#[component]
fn MeasureBlock(label: String, value: String) -> Element {
    rsx! {
        div { class: "pdsa-measure-block",
            span { "{label}" }
            strong {
                if value.trim().is_empty() {
                    "Not set"
                } else {
                    "{value}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PdsaTextField {
    Plan,
    Prediction,
    Do,
    Study,
    Act,
}

#[component]
fn CycleTextArea(id: String, label: String, value: String, field: PdsaTextField) -> Element {
    let mut pdsa = use_pdsa();
    rsx! {
        label { class: "pdsa-field pdsa-field-wide",
            span { "{label}" }
            textarea {
                value: "{value}",
                oninput: move |e| {
                    let next = e.value();
                    update_cycle(&mut pdsa, &id, |cycle| match field {
                        PdsaTextField::Plan => cycle.plan = next.clone(),
                        PdsaTextField::Prediction => cycle.prediction = next.clone(),
                        PdsaTextField::Do => cycle.doing = next.clone(),
                        PdsaTextField::Study => cycle.study = next.clone(),
                        PdsaTextField::Act => cycle.act = next.clone(),
                    });
                },
            }
        }
    }
}

fn update_cycle(
    pdsa: &mut Signal<crate::state::PdsaState>,
    id: &str,
    apply: impl FnOnce(&mut PdsaCycle),
) {
    {
        let mut state = pdsa.write();
        if let Some(cycle) = state.cycles.iter_mut().find(|cycle| cycle.id == id) {
            apply(cycle);
            cycle.updated_at = crate::today_iso();
        }
    }
    crate::persist_pdsa_state(&pdsa.read());
}

fn next_cycle_id(title: &str, count: usize, date: &str) -> String {
    let slug = slugify(title);
    format!("pdsa-{date}-{count}-{slug}")
}

fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "cycle".to_string()
    } else {
        out
    }
}
