use dioxus::prelude::*;

use crate::state::use_archive;

const PHASE_COLORS: &[(&str, &str)] = &[
    ("Antecedents", "#6b7280"),
    ("Convention", "#2563eb"),
    ("Ratification", "#16a34a"),
    ("Bill of Rights Drafting", "#d97706"),
    ("Bill of Rights Ratification", "#c4452f"),
];

fn phase_color(label: &str) -> &'static str {
    PHASE_COLORS
        .iter()
        .find(|(l, _)| *l == label)
        .map(|(_, c)| *c)
        .unwrap_or("#888")
}

#[component]
pub fn TimelineSvg(selected_key: Option<String>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let timeline = match state.archive.as_ref() {
        Some(a) => a.timeline(),
        None => return rsx! { p { "No timeline data." } },
    };

    if timeline.events.is_empty() {
        return rsx! { p { "No timeline events." } };
    }

    let width = 900.0_f64;
    let height = 120.0_f64;
    let margin = 40.0_f64;
    let usable = width - 2.0 * margin;

    let phases = state
        .archive
        .as_ref()
        .map(|a| a.timeline_by_phase())
        .unwrap_or_default();

    let total_events: usize = phases.values().map(|v| v.len()).sum();
    if total_events == 0 {
        return rsx! { p { "No timeline events." } };
    }

    let mut x_offset = margin;
    let phase_widths: Vec<(&str, f64, f64, usize)> = phases
        .iter()
        .map(|(label, events)| {
            let w = usable * (events.len() as f64 / total_events as f64);
            let start = x_offset;
            x_offset += w;
            (*label, start, w, events.len())
        })
        .collect();

    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            class: "timeline-svg",

            // Phase bands
            for &(label, start, w, count) in phase_widths.iter() {
                {
                    let color = phase_color(label);
                    rsx! {
                        rect {
                            x: "{start}",
                            y: "20",
                            width: "{w}",
                            height: "40",
                            fill: "{color}",
                            opacity: "0.15",
                            rx: "4",
                        }
                        text {
                            x: "{start + w / 2.0}",
                            y: "15",
                            text_anchor: "middle",
                            font_size: "10",
                            fill: "{color}",
                            "{label}"
                        }
                        text {
                            x: "{start + w / 2.0}",
                            y: "80",
                            text_anchor: "middle",
                            font_size: "9",
                            fill: "#666",
                            "{count} events"
                        }
                    }
                }
            }

            // Center line
            line {
                x1: "{margin}",
                y1: "40",
                x2: "{width - margin}",
                y2: "40",
                stroke: "#ccc",
                stroke_width: "1",
            }

            // Event dots
            {
                let mut elements = Vec::new();
                for (label, start, w, _count) in &phase_widths {
                    let events = phases.get(*label).cloned().unwrap_or_default();
                    let n = events.len();
                    for (i, _event) in events.iter().enumerate() {
                        let x = if n <= 1 {
                            start + w / 2.0
                        } else {
                            start + 4.0 + (w - 8.0) * (i as f64 / (n - 1) as f64)
                        };
                        elements.push((x, phase_color(label)));
                    }
                }
                rsx! {
                    for (x, color) in elements.iter() {
                        circle {
                            cx: "{x}",
                            cy: "40",
                            r: "3",
                            fill: "{color}",
                        }
                    }
                }
            }
        }

        // Legend
        div { class: "timeline-legend",
            for &(label, color) in PHASE_COLORS {
                span { class: "legend-item",
                    span {
                        class: "legend-dot",
                        style: "background: {color};",
                    }
                    "{label}"
                }
            }
        }
    }
}
