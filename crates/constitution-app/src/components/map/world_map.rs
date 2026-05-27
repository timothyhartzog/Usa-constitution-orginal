use dioxus::prelude::*;

use crate::state::use_archive;

const REGION_COLORS: &[(&str, &str)] = &[
    ("Africa", "#c4452f"),
    ("Americas", "#2563eb"),
    ("Asia", "#d97706"),
    ("Europe", "#16a34a"),
    ("Oceania", "#7c3aed"),
];

fn region_color(region: &str) -> &'static str {
    REGION_COLORS
        .iter()
        .find(|(r, _)| *r == region)
        .map(|(_, c)| *c)
        .unwrap_or("#888")
}

#[component]
pub fn WorldMapSvg(selected_country: Option<String>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let counts_by_region: Vec<(String, usize)> = {
        let mut map = std::collections::HashMap::new();
        for m in &state.world_meta {
            *map.entry(m.region.clone()).or_insert(0) += 1;
        }
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };

    rsx! {
        div { class: "world-map-container",
            svg {
                width: "700",
                height: "360",
                view_box: "0 0 700 360",
                class: "world-map-svg",

                rect {
                    x: "0", y: "0", width: "700", height: "360",
                    fill: "#f0f4f8", rx: "8",
                }

                // Region bubbles as a cartogram placeholder
                {
                    let positions: &[(&str, f64, f64)] = &[
                        ("Africa", 380.0, 200.0),
                        ("Americas", 150.0, 180.0),
                        ("Asia", 520.0, 140.0),
                        ("Europe", 380.0, 80.0),
                        ("Oceania", 580.0, 280.0),
                    ];
                    rsx! {
                        for &(region, cx, cy) in positions {
                            {
                                let count = counts_by_region
                                    .iter()
                                    .find(|(r, _)| r == region)
                                    .map(|(_, c)| *c)
                                    .unwrap_or(0);
                                let r = 15.0 + (count as f64).sqrt() * 8.0;
                                let color = region_color(region);
                                rsx! {
                                    circle {
                                        cx: "{cx}",
                                        cy: "{cy}",
                                        r: "{r}",
                                        fill: "{color}",
                                        opacity: "0.3",
                                    }
                                    text {
                                        x: "{cx}",
                                        y: "{cy - 2.0}",
                                        text_anchor: "middle",
                                        font_size: "12",
                                        font_weight: "600",
                                        fill: "{color}",
                                        "{region}"
                                    }
                                    text {
                                        x: "{cx}",
                                        y: "{cy + 12.0}",
                                        text_anchor: "middle",
                                        font_size: "10",
                                        fill: "#666",
                                        "{count} constitutions"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Legend
            div { class: "map-legend",
                for &(region, color) in REGION_COLORS {
                    span { class: "legend-item",
                        span {
                            class: "legend-dot",
                            style: "background: {color};",
                        }
                        "{region}"
                    }
                }
            }
        }
    }
}
